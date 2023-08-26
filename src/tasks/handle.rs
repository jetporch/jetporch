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

use crate::connection::connection::Connection;
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskStatus, TaskResponse};
use crate::tasks::logic::{PreLogicEvaluated,PostLogicEvaluated};
use crate::inventory::hosts::{Host,HostOSType};
use std::collections::HashSet;
use std::sync::{Arc,Mutex,RwLock};
use crate::playbooks::traversal::RunState;
use crate::connection::command::{CommandResult,cmd_info};
use std::path::{Path,PathBuf};
use crate::tasks::fields::Field;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;

// task handles are given to modules to give them shortcuts to work with the jet system
// actual functionality is mostly provided via TaskRequest/TaskResponse and such, the handles
// are mostly module authors don't need to think about how things work as much.  This is
// especially true for the finite state machine that executes tasks.

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

    // ================================================================================
    // PLAYBOOK UTILS: simplified interactions to make module code nicer.

    #[inline]
    pub fn run(&self, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        return self.connection.lock().unwrap().run_command(self, request, cmd);
    }

    pub fn get_os_type(&self) -> HostOSType {
        let os_type = self.host.read().unwrap().os_type;
        if os_type.is_none() {
            panic!("failed to detect OS type for {}, bailing out", self.host.read().unwrap().name);
        }
        return os_type.unwrap();
    }

    pub fn remote_stat(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Option<String>,Arc<TaskResponse>> {
        let cmd : String = crate::tasks::cmd_library::get_mode_command(self.get_os_type(), path);

        let result = self.run(request,&cmd)?;
        let (rc, out) = cmd_info(&result);
        return match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => Ok(Some(out.split_whitespace().nth(0).unwrap().to_string())),
            _ => Ok(None),
        }
    }

    #[inline]
    pub fn find_template_path(&self, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        return self.find_sub_path(&String::from("templates"), request, field, str_path);
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
    pub fn debug_lines(&self, request: &Arc<TaskRequest>, messages: &Vec<String>) {
        self.run_state.visitor.read().unwrap().debug_lines(&Arc::clone(&self.run_state.context), &self.host, messages);
    }

    /*
    pub fn query_file_attributes(&self, request: &ArcTaskRequest, remote_path: &String, stat_result: &Option<String>, input_checksum: Option<String>, changes: &mut HashSet<String>) {

        if stat_result.is_none() {
            return Err(handle.is_failed(request, String::from("module coding error: calling query_file_attributes with no remote file")));
        }

        if input_checksum.is_some() {
            checksum_src = self.string_checksum(request, remote_path)?;
            checksum_dest = handle.remote_checksum(request, self.dest)?;
            if checksum_src != checksum_dest {
                changes.push(String::from("dest"));
            }
        }
    
    if self.attributes.is_some() {
        let attributes = self.attributes.unwrap();
        let owner = handle.remote_owner(self.dest)?
        if attributes.owner.is_some() {
            let owner = handle.remote_owner(self.dest)?;
            if (owner != attributes.owner.unwrap()) { changes.push(String::from("owner")); }
        }
        if attributes.group.is_some()
            let owner = handle.remote_group(self.dest)?;
            if (group != attributes.owner.group())  { changes.push(String::from("group")); }
        }
        if attributes.mode.is_some() {
            if (stat_result.unwrap() != attributes.owner.mode) { changes.push(String::from("mode")); }
        }
    }
    */

    pub fn template_string(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        return match result {
            Ok(x) => Ok(x),
            Err(y) => {
                Err(self.is_failed(request, &y))
            }
        }
    }

    pub fn template_string_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        if template.is_none() { return Ok(None); }
        let result = self.template_string(request, field, &template.as_ref().unwrap());
        return match result { 
            Ok(x) => Ok(Some(x)), 
            Err(y) => {
                Err(self.is_failed(request, &format!("field ({}) template error: {:?}", field, y)))
            } 
        };
    }

    pub fn template_integer(&self, request: &Arc<TaskRequest>, field: &String, template: &String)-> Result<i64,Arc<TaskResponse>> {
        let st = self.template_string(request, field, template)?;
        let num = st.parse::<i64>();
        match num {
            Ok(num) => Ok(num),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_integer_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<i64>,Arc<TaskResponse>> {
        if template.is_none() { return Ok(None); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let num = st.parse::<i64>();
        match num {
            Ok(num) => Ok(Some(num)),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_boolean(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<bool,Arc<TaskResponse>> {
        let st = self.template_string(request,field, template)?;
        let x = st.parse::<bool>();
        match x {
            Ok(x) => Ok(x),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn template_boolean_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        if template.is_none() { return Ok(false); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        match x {
            Ok(x) => Ok(x),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn test_cond(&self, request: &Arc<TaskRequest>, expr: &String) -> Result<bool, Arc<TaskResponse>> {
        let result = self.get_context().read().unwrap().test_cond(expr, &self.host);
        return match result {
            Ok(x) => Ok(x),
            Err(y) => Err(self.is_failed(request, &y))
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
        // FIXME: use the task result
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
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsSkipped, 
            changes: HashSet::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
        return response;
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
            status: TaskStatus::IsExecuted, 
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
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: HashSet<Field>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: HashSet::new(), 
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