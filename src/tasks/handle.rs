// Jetporch
// Copyright (C) 2023 - Michael DeHaan <michael@michaeldehaan.net> + contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// long with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::HashSet;
use std::sync::{Arc,Mutex,RwLock};
use std::path::{Path,PathBuf};

use crate::connection::connection::Connection;
use crate::connection::command::{CommandResult,cmd_info};
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskStatus, TaskResponse};
use crate::inventory::hosts::{Host,HostOSType};
use crate::playbooks::traversal::RunState;
use crate::tasks::fields::Field;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::tasks::FileAttributesEvaluated;
use crate::tasks::cmd_library::screen_path;

// task handles are given to modules to give them shortcuts to work with the jet system
// actual functionality is mostly provided via TaskRequest/TaskResponse and such, the handles
// are mostly module authors don't need to think about how things work as much.  This is
// especially true for the finite state machine that executes tasks.

#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum LocalRemote {
    Local,
    Remote
}

pub struct TaskHandle {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>,
}

impl TaskHandle {

    pub fn new(run_state_handle: Arc<RunState>, connection_handle: Arc<Mutex<dyn Connection>>, host_handle: Arc<RwLock<Host>>) -> Self {
        Self {
            run_state: run_state_handle,
            connection: connection_handle,
            host: host_handle,
        }
    }

    #[inline]
    pub fn get_context(&self) -> Arc<RwLock<PlaybookContext>> {
        return Arc::clone(&self.run_state.context);
    }

    #[inline]
    pub fn get_visitor(&self) -> Arc<RwLock<dyn PlaybookVisitor>> {
        return Arc::clone(&self.run_state.visitor);
    }

    #[inline]
    pub fn run(&self, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        return self.connection.lock().unwrap().run_command(self, request, cmd);
    }

    fn run_local(&self, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type == TaskRequestType::Query, "local commands can only be run in query stage (was: {:?})", request.request_type);
        let ctx = &self.run_state.context;
        let local_result = self.run_state.connection_factory.read().unwrap().get_local_connection(&ctx);
        let local_conn = match local_result {
            Ok(x) => x,
            Err(y) => { return Err(self.is_failed(request, &y.clone())) }
        };
        return local_conn.lock().unwrap().run_command(self, request, cmd);
    }

    pub fn get_os_type(&self) -> HostOSType {
        let os_type = self.host.read().unwrap().os_type;
        if os_type.is_none() {
            panic!("failed to detect OS type for {}, bailing out", self.host.read().unwrap().name);
        }
        return os_type.unwrap();
    }

    pub fn read_local_file(&self, request: &Arc<TaskRequest>, path: &Path) -> Result<String, Arc<TaskResponse>> {
        return match crate::util::io::read_local_file(path) {
            Ok(s) => Ok(s),
            Err(x) => Err(self.is_failed(request, &x.clone()))
        };
    }
    
    // writes a string (for example, from a template) to a remote file location
    pub fn write_remote_data(&self, request: &Arc<TaskRequest>, data: &String, path: &String, mode: Option<i32>) -> Result<(), Arc<TaskResponse>> {
        return self.connection.lock().unwrap().write_data(self, &request, &data.clone(), &path.clone(), mode);
    }

    fn unwrap_string_result(&self, request: &Arc<TaskRequest>, str_result: &Result<String,String>) -> Result<String, Arc<TaskResponse>> {
        return match str_result {
            Ok(x) => Ok(x.clone()),
            Err(y) => Err(self.is_failed(request, &y.clone()))
        };
    }

    pub fn get_remote_mode(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Option<String>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_mode_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        
        let result = self.run(request,&cmd)?;
        let (rc, out) = cmd_info(&result);
        return match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => Ok(Some(out.split_whitespace().nth(0).unwrap().to_string())),
            _ => Ok(None),
        }
    }
    
    pub fn get_remote_ownership(&self, request: &Arc<TaskRequest>, path: &String) -> Result<(String,String),Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_ownership_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        
        let result = self.run(request,&cmd)?;
        let (rc, out) = cmd_info(&result);

        match rc {
            0 => {
                let mut split = out.split_whitespace();
                let owner = match split.nth(2) {
                    Some(x) => x,
                    None => { return Err(self.is_failed(request, &format!("unexpected output format from {}: {}", cmd, out))) }
                };
                let group = match split.nth(3) {
                    Some(x) => x,
                    None => { return Err(self.is_failed(request, &format!("unexpected output format from {}: {}", cmd, out))) }
                };
                return Ok((owner.to_string(),group.to_string()))
            },
            _ => {
                return Err(self.is_failed(request, &format!("ls failed, rc: {}: {}", rc, out)));
            }
        }
    }

    pub fn get_localhost(&self) -> Arc<RwLock<Host>> {
        let inventory = self.run_state.inventory.read().unwrap();
        return inventory.get_host(&String::from("localhost"));
    }

    pub fn get_local_sha512(&self, request: &Arc<TaskRequest>, path: &Path, use_cache: bool) -> Result<String,Arc<TaskResponse>> {
        let path2 = format!("{}", path.display());
        let localhost = self.get_localhost();
        if use_cache {
            let ctx = self.run_state.context.read().unwrap();
            let task_id = ctx.get_task_count();
            let mut localhost2 = localhost.write().unwrap();
            let cached = localhost2.get_checksum_cache(task_id, &path2);
            if cached.is_some() {
                return Ok(cached.unwrap());
            }
        }
        let value = self.internal_remote_sha512(request, LocalRemote::Local, &path2)?;
        if use_cache {
            let mut localhost2 = localhost.write().unwrap();
            localhost2.set_checksum_cache(&path2, &value);
        }
        return Ok(value);
    }

    pub fn get_remote_sha512(&self, request: &Arc<TaskRequest>, path: &String) -> Result<String,Arc<TaskResponse>> {
        return self.internal_remote_sha512(request, LocalRemote::Remote, path);
    }
   

    pub fn internal_remote_sha512(&self, request: &Arc<TaskRequest>, is_local: LocalRemote, path: &String) -> Result<String,Arc<TaskResponse>> {
        
        // these games around local command execution should only happen here and not complicate other functions, 
        // local_action/delegate_to will happen at a higher level in the taskfsm.

        let get_cmd_result = match is_local {
            LocalRemote::Local => {
                let localhost = self.get_localhost();
                let os_type = localhost.read().unwrap().os_type.expect("unable to detect host OS type");
                crate::tasks::cmd_library::get_sha512_command(os_type, path)
            },
            LocalRemote::Remote => {
                let os_type = self.get_os_type();
                crate::tasks::cmd_library::get_sha512_command(os_type, path)
            }
        };
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;

        let result = match is_local {
            LocalRemote::Remote => self.run(request, &cmd)?,
            LocalRemote::Local  => self.run_local(request, &cmd)?
        };

        let (rc, out) = cmd_info(&result);
        match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => {
                let value = out.split_whitespace().nth(0).unwrap().to_string();
                return Ok(value);
            },
            127 => {
                // file not found
                return Ok(String::from(""))
            },
            _ => {
                return Err(self.is_failed(request, &format!("checksum failed: {}. {}", path, out)));
            }
        };
    }

    pub fn get_desired_numeric_mode(&self, request: &Arc<TaskRequest>, attribs: &Option<FileAttributesEvaluated>) -> Result<Option<i32>,Arc<TaskResponse>>{
        return FileAttributesEvaluated::get_numeric_mode(self, request, attribs); 
    }

    #[inline]
    pub fn find_template_path(&self, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        // source path variables cannot use templates
        // we also screen them for the same invalid characters we check for in dest paths
        let prelim = match screen_path(&str_path) {
            Ok(x) => x, Err(y) => { return Err(self.is_failed(request, &format!("{}, for field: {}", y, field))) }
        };
        return self.find_sub_path(&String::from("templates"), request, field, &prelim);
    }

    fn find_sub_path(&self, prefix: &String, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        let mut path = PathBuf::new();
        path.push(str_path);
        if path.is_absolute() {
            if path.is_file() {
                return Ok(path);
            } else {
                return Err(self.is_failed(request, &format!("field ({}): no such file: {}", field, str_path)));
            }
        } else {
            let mut path2 = PathBuf::new();
            path2.push(prefix);
            path2.push(str_path);
            if path2.is_file() {
                return Ok(path2);
            } else {
                return Err(self.is_failed(request, &format!("field field ({}): no such file: {}", field, str_path)));
            }
        }
    }

    #[inline]
    pub fn debug(&self, _request: &Arc<TaskRequest>, message: &String) {
        self.run_state.visitor.read().unwrap().debug_host(&self.host, message);
    }

    #[inline]
    pub fn debug_lines(&self, _request: &Arc<TaskRequest>, messages: &Vec<String>) {
        self.run_state.visitor.read().unwrap().debug_lines(&Arc::clone(&self.run_state.context), &self.host, messages);
    }

    pub fn query_common_file_attributes(&self, request: &Arc<TaskRequest>, remote_path: &String, 
        attributes_in: &Option<FileAttributesEvaluated>, changes: &mut HashSet<Field>) -> Result<Option<String>,Arc<TaskResponse>> {
        let remote_mode = self.get_remote_mode(request, remote_path)?;
        if remote_mode.is_none() {
            return Ok(None);
        }
        if attributes_in.is_some() {
            let attributes = attributes_in.as_ref().unwrap();
            let (remote_owner, remote_group) = self.get_remote_ownership(request, remote_path)?;
            if attributes.owner.is_some() {
                if ! remote_owner.eq(attributes.owner.as_ref().unwrap()) { 
                    println!("owner change!");
                    changes.insert(Field::Owner); 
                }
            }
            if attributes.group.is_some() {
                if ! remote_group.eq(attributes.group.as_ref().unwrap())  { 
                    println!("group change!");
                    changes.insert(Field::Group); 
                }
            }
            if attributes.mode.is_some() {
                if ! remote_mode.as_ref().unwrap().eq(attributes.mode.as_ref().unwrap()) { 
                    println!("mode change!");
                    changes.insert(Field::Mode); 
                }
            }
            // FIXME: other common attributes like SELinux would go here = tasks/files.rs
        }
 
        return Ok(remote_mode);
    }

    pub fn template_string(&self, request: &Arc<TaskRequest>, _field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // note to module authors:
        // if you have a path, call template_path instead!  Do not call template_str as you will ignore path sanity checks.
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        let result2 = self.unwrap_string_result(request, &result)?;
        return Ok(result2);
    }

    pub fn template_path(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        let result2 = self.unwrap_string_result(request, &result)?;
        return match screen_path(&result2) {
            Ok(x) => Ok(x), Err(y) => { return Err(self.is_failed(request, &format!("{}, for field {}", y, field))) }
        }
    }

    pub fn template_string_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // note to module authors:
        // if you have a path, call template_path instead!  Do not call template_str as you will ignore path sanity checks.
        if template.is_none() { return Ok(None); }
        let result = self.template_string(request, field, &template.as_ref().unwrap());
        return match result { 
            Ok(x) => Ok(Some(x)), Err(y) => { Err(self.is_failed(request, &format!("field ({}) template error: {:?}", field, y))) } 
        };
    }

    pub fn template_integer(&self, request: &Arc<TaskRequest>, field: &String, template: &String)-> Result<i64,Arc<TaskResponse>> {
        let st = self.template_string(request, field, template)?;
        let num = st.parse::<i64>();
        return match num {
            Ok(num) => Ok(num), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_integer_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<i64>,Arc<TaskResponse>> {
        if template.is_none() { return Ok(None); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let num = st.parse::<i64>();
        // FIXME: these can use map_err
        return match num {
            Ok(num) => Ok(Some(num)), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_boolean(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<bool,Arc<TaskResponse>> {
        let st = self.template_string(request,field, template)?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn template_boolean_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        if template.is_none() { return Ok(false); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn test_cond(&self, request: &Arc<TaskRequest>, expr: &String) -> Result<bool, Arc<TaskResponse>> {
        let result = self.get_context().read().unwrap().test_cond(expr, &self.host);
        return match result {
            Ok(x) => Ok(x), Err(y) => Err(self.is_failed(request, &y))
        }
    }

    // ================================================================================
    // RETURN WRAPPERS FOR EVERY TASK REQUEST TYPE

    pub fn is_failed(&self, _request: &Arc<TaskRequest>,  msg: &String) -> Arc<TaskResponse> {
        return Arc::new(TaskResponse { 
            status: TaskStatus::Failed, 
            changes: HashSet::new(), 
            msg: Some(msg.clone()), 
            command_result: Arc::new(None), 
            with: Arc::new(None), 
            and: Arc::new(None)
        });
    }

    #[inline]
    pub fn not_supported(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        return self.is_failed(request, &String::from("not supported"));
    }

    pub fn command_failed(&self, _request: &Arc<TaskRequest>, result: &Arc<Option<CommandResult>>) -> Arc<TaskResponse> {
        self.get_visitor().read().expect("read visitor").on_command_failed(&self.get_context(), &Arc::clone(&self.host), &Arc::clone(result));
        return Arc::new(TaskResponse {
            status: TaskStatus::Failed,
            changes: HashSet::new(), 
            msg: Some(String::from("command failed")), 
            command_result: Arc::clone(&result), 
            with: Arc::new(None), 
            and: Arc::new(None)
        });
    }

    pub fn command_ok(&self, _request: &Arc<TaskRequest>, result: &Arc<Option<CommandResult>>) -> Arc<TaskResponse> {
        self.get_visitor().read().expect("read visitor").on_command_ok(&self.get_context(), &Arc::clone(&self.host), &Arc::clone(result));
        return Arc::new(TaskResponse {
            status: TaskStatus::IsExecuted,
            changes: HashSet::new(), msg: None, command_result: Arc::clone(&result), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_skipped(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_skipped response can only be returned for a validation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsSkipped, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_matched(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "is_matched response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsMatched, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Create, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsCreated, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    // see also command_ok for shortcuts, as used in the shell module.
    pub fn is_executed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Execute, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Remove, "is_removed response can only be returned for a remove request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsRemoved, 
            changes: HashSet::new(), 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Passive, "is_passive response can only be returned for a passive request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsPassive, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: HashSet<Field>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Modify, "is_modified response can only be returned for a modification request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsModified, 
            changes: changes, 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_creation response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsCreation, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None), 
        });
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: &HashSet<Field>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        assert!(!changes.is_empty(), "changes must not be empty");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: changes.clone(), 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None) 
        });
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_removal response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_execution(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_execution response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsExecution, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None),and: Arc::new(None)
        });
    }
    
    pub fn needs_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_passive response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsPassive, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

}