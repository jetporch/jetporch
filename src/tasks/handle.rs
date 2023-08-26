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
//use crate::tasks::logic::{PreLogicEvaluated,PostLogicEvaluated};
use crate::inventory::hosts::{Host,HostOSType};
use std::collections::HashSet;
use std::sync::{Arc,Mutex,RwLock};
use crate::playbooks::traversal::RunState;
use crate::connection::command::{CommandResult,cmd_info};
use std::path::{Path,PathBuf};
use crate::tasks::fields::Field;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::util::io::jet_file_open;
//use std::fs;
use std::io::Read;
use crate::tasks::FileAttributesEvaluated;

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

    // returns the context object. Generally module callers shouldn't need to get a hold of this one, but it's a useful place
    // to grab a write lock when sending multi-line output.
    #[inline]
    pub fn get_context(&self) -> Arc<RwLock<PlaybookContext>> {
        return Arc::clone(&self.run_state.context);
    }

    // this visitor is useful for callbacks, though most are handled for the modules by task FSM.  Use sparingly from module
    // code.
    #[inline]
    pub fn get_visitor(&self) -> Arc<RwLock<dyn PlaybookVisitor>> {
        return Arc::clone(&self.run_state.visitor);
    }

    // ================================================================================
    // PLAYBOOK UTILS: simplified interactions to make module code nicer.

    // runs an external command and returns a task response
    // return code and output can be easily extracted by using the cmd_info() function on a task response
    // we know has command info in it.
    #[inline]
    pub fn run(&self, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        return self.connection.lock().unwrap().run_command(self, request, cmd);
    }

    // reads the host OS type which is determined at connection time
    pub fn get_os_type(&self) -> HostOSType {
        let os_type = self.host.read().unwrap().os_type;
        if os_type.is_none() {
            panic!("failed to detect OS type for {}, bailing out", self.host.read().unwrap().name);
        }
        return os_type.unwrap();
    }

    // get the string contents of a local file, this is not really for copy operations but for small files like template sources
    pub fn read_local_file(&self, request: &Arc<TaskRequest>, path: &Path) -> Result<String, Arc<TaskResponse>> {
        let mut file = jet_file_open(path);
        match file {
            Ok(mut f) => {
                let mut buffer = String::new();
                let read_result = f.read_to_string(&mut buffer);
                match read_result {
                    Ok(_) => {},
                    Err(x) => {
                        return Err(self.is_failed(&request, &format!("unable to read file: {}, {:?}", path.display(), x)));
                    }
                };
                return Ok(buffer.clone());
            }
            Err(x) => {
                return Err(self.is_failed(&request, &format!("unable to open file: {}, {:?}", path.display(), x)));
            }
        };
    }
    
    // writes a string (for example, from a template) to a remote file location
    pub fn write_remote_data(&self, request: &Arc<TaskRequest>, data: &String, path: &String, mode: Option<i32>) -> Result<(), Arc<TaskResponse>> {
        return self.connection.lock().unwrap().write_data(self, &request, &data.clone(), &path.clone(), mode);
    }

    // gets the mode of a remote file as an octal string with no prefix
    pub fn get_remote_mode(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Option<String>,Arc<TaskResponse>> {
        let cmd : String = crate::tasks::cmd_library::get_mode_command(self.get_os_type(), path);

        let result = self.run(request,&cmd)?;
        let (rc, out) = cmd_info(&result);
        return match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => Ok(Some(out.split_whitespace().nth(0).unwrap().to_string())),
            _ => Ok(None),
        }
    }

    // gets the numeric octal value of the task object (not to be used with remote files)
    pub fn get_desired_numeric_mode(&self, request: &Arc<TaskRequest>, attribs: &Option<FileAttributesEvaluated>) -> Result<Option<i32>,Arc<TaskResponse>>{
        return FileAttributesEvaluated::get_numeric_mode(self, request, attribs); 
    }

    // given a field name and a path fragment, find a file in where it should normally be found ('./templates')
    #[inline]
    pub fn find_template_path(&self, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        return self.find_sub_path(&String::from("templates"), request, field, str_path);
    }

    // supporting code for functions like find_template_path
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

    // outputs a debug message to the screen, note that this ignores verbosity levels and is really here
    // to support modules whose primary purpose is debugging, like echo, and not module programming, where
    // a temporary println may be more appropriate.
    #[inline]
    pub fn debug(&self, _request: &Arc<TaskRequest>, message: &String) {
        self.run_state.visitor.read().unwrap().debug_host(&self.host, message);
    }

    // similar to debug_lines but acquires a lock for multi-line output that will not interlace
    // with parallel SSH requests.
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

    // renders a template with variables from the current host (and everything else in the system)
    // this is used for evaluating fields as well as template operations for the template module
    pub fn template_string(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        return match result {
            Ok(x) => Ok(x),
            Err(y) => {
                Err(self.is_failed(request, &y))
            }
        }
    }

    // this is used to evaluate fields that have optional string values
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

    // this is used to template fields that render down into integers
    pub fn template_integer(&self, request: &Arc<TaskRequest>, field: &String, template: &String)-> Result<i64,Arc<TaskResponse>> {
        let st = self.template_string(request, field, template)?;
        let num = st.parse::<i64>();
        match num {
            Ok(num) => Ok(num),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    // this is used to template fields that render down into integers, but can be omitted
    pub fn template_integer_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<i64>,Arc<TaskResponse>> {
        if template.is_none() { return Ok(None); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let num = st.parse::<i64>();
        match num {
            Ok(num) => Ok(Some(num)),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    // this is used to template fields that must render down to booleans.
    pub fn template_boolean(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<bool,Arc<TaskResponse>> {
        let st = self.template_string(request,field, template)?;
        let x = st.parse::<bool>();
        match x {
            Ok(x) => Ok(x),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    // this is used to template fields that must render down to booleans but are optional
    pub fn template_boolean_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        if template.is_none() { return Ok(false); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        match x {
            Ok(x) => Ok(x),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    // evaluates a conditional expression - this is for "with/cond" statements.
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