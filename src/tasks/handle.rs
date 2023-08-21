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
use crate::tasks::response::{TaskResponse, TaskStatus};
use crate::inventory::hosts::Host;
use std::collections::HashMap;
use std::sync::{Arc,Mutex,RwLock};
use crate::playbooks::traversal::RunState;
use crate::connection::command::CommandResult;
use once_cell::sync::Lazy;

pub struct TaskHandle {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>,
}


static OUTPUT_LOCK: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

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
        // FIXME should visitor debug take a reference?
        self.run_state.visitor.read().unwrap().debug_host(&self.host, message.clone());
    }

    pub fn debug_lines(&self, request: &Arc<TaskRequest>, messages: &Vec<String>) {
        OUTPUT_LOCK.lock();
        // this later may be modified to acquire a lock and keep messages together
        for message in messages.iter() {
            self.debug(request, &message);
        }
    }



    // ================================================================================
    // RETURN WRAPPERS FOR EVERY TASK REQUEST TYPE

    pub fn is_failed(&self, _request: &Arc<TaskRequest>,  msg: &String) -> Arc<TaskResponse> {
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::Failed, 
            changes: Arc::new(None),
            msg: Some(msg.clone()), 
            command_result: None
        });
        // FIXME: make a function for this
        return response;
    }

    pub fn not_supported(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        return self.is_failed(request, &String::from("not supported"));
    }

    pub fn command_failed(&self, _request: &Arc<TaskRequest>, result: CommandResult) -> Arc<TaskResponse> {
        let response = Arc::new(TaskResponse {
            status: TaskStatus::Failed,
            changes: Arc::new(None),
            msg: None,
            command_result: Some(result)
        });
        return response;
    }

    pub fn command_ok(&self, _request: &Arc<TaskRequest>, result: CommandResult) -> Arc<TaskResponse> {
        let response = Arc::new(TaskResponse {
            status: TaskStatus::IsExecuted,
            changes: Arc::new(None),
            msg: None,
            command_result: Some(result)
        });
        return response;
    }

    pub fn is_validated(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_validated response can only be returned for a validation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsValidated, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None
        });
        return response;
    }

    pub fn is_matched(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "is_matched response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsMatched, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None
        });
        return response;
    }

    // FIXME: reduce boilerplate in all of these...

    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Create, "is_executed response can only be returned for a creation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    // see also command_ok for shortcuts, as used in the shell module.
    pub fn is_executed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Execute, "is_executed response can only be returned for a creation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Remove, "is_removed response can only be returned for a remove request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsRemoved, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }

    pub fn is_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Passive, "is_passive response can only be returned for a passive request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsPassive, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Modify, "is_modified response can only be returned for a modification request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsModified, 
            changes: Arc::clone(&changes), 
            msg: None,
            command_result: None 
        });
        return response;
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_creation response can only be returned for a query request");

        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsCreation, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: Arc::clone(&changes), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_removal response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }

    pub fn needs_execution(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_execution response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsExecution, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }
    
    pub fn needs_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_passive response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsPassive, 
            changes: Arc::new(None), 
            msg: None,
            command_result: None 
        });
        return response;
    }

}