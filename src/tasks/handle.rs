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
use crate::inventory::hosts::Host;
use std::collections::HashMap;
use std::sync::{Arc,Mutex,RwLock};
use crate::playbooks::traversal::RunState;
use crate::connection::command::CommandResult;

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

    // ================================================================================
    // CONNECTION INTERACTION

    pub fn run(&self, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        return self.connection.lock().unwrap().run_command(self, request, cmd);
    }

    // ================================================================================
    // PLAYBOOK INTERACTION: simplified interactions with the visitor object
    // to make module code nicer.

    pub fn debug(&self, _request: &Arc<TaskRequest>, message: &String) {
        self.run_state.visitor.read().unwrap().debug_host(&self.host, message);
    }

    pub fn debug_lines(&self, request: &Arc<TaskRequest>, messages: &Vec<String>) {
        self.run_state.visitor.read().unwrap().debug_lines(&Arc::clone(&self.run_state.context), &self.host, messages);
    }

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
        if template.is_none() {
            return Ok(None);
        }
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
        if template.is_none() {
            return Ok(None);
        }
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
        if template.is_none() {
            return Ok(false);
        }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        match x {
            Ok(x) => Ok(x),
            Err(err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn test_cond(&self, request: &Arc<TaskRequest>, expr: &String) -> Result<bool, Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().test_cond(expr, &self.host);
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
            changes: Arc::new(None), msg: Some(msg.clone()), command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn not_supported(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        return self.is_failed(request, &String::from("not supported"));
    }

    pub fn command_failed(&self, _request: &Arc<TaskRequest>, result: CommandResult) -> Arc<TaskResponse> {
        return Arc::new(TaskResponse {
            status: TaskStatus::Failed,
            changes: Arc::new(None), msg: Some(String::from("command failed")), command_result: Some(result), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn command_ok(&self, _request: &Arc<TaskRequest>, result: CommandResult) -> Arc<TaskResponse> {
        return Arc::new(TaskResponse {
            status: TaskStatus::IsExecuted,
            changes: Arc::new(None), msg: None, command_result: Some(result), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_validated(&self, request: &Arc<TaskRequest>, with: &Arc<Option<PreLogicEvaluated>>, and: &Arc<Option<PostLogicEvaluated>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_validated response can only be returned for a validation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsValidated, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::clone(with), and: Arc::clone(and)
        });
    }

    pub fn is_skipped(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_skipped response can only be returned for a validation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsSkipped, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
        return response;
    }

    pub fn is_matched(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "is_matched response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsMatched, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Create, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    // see also command_ok for shortcuts, as used in the shell module.
    pub fn is_executed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Execute, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Remove, "is_removed response can only be returned for a remove request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsRemoved, 
            changes: Arc::new(None), 
            msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Passive, "is_passive response can only be returned for a passive request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsPassive, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Modify, "is_modified response can only be returned for a modification request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsModified, 
            changes: Arc::clone(&changes), 
            msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_creation response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsCreation, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None), 
        });
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: Arc::clone(&changes), 
            msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None) 
        });
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_removal response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_execution(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_execution response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsExecution, 
            changes: Arc::new(None), msg: None, command_result: None,with: Arc::new(None),and: Arc::new(None)
        });
    }
    
    pub fn needs_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_passive response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsPassive, 
            changes: Arc::new(None), msg: None, command_result: None, with: Arc::new(None), and: Arc::new(None)
        });
    }

}