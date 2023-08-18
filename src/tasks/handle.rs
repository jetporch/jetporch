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

// ===================================================================================
// ABOUT: task_handle.rs
// a task handle warps lots of playbook reporting, connection, and command details
// to help ensure a module does not have too much API access to the rest of the program
// and mostly standardized
// ===================================================================================

//use crate::playbooks::context::PlaybookContext;
//use crate::playbooks::visitor::PlaybookVisitor;
use crate::connection::connection::Connection;
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskResponse, TaskStatus};
use crate::connection::command::Command;
//use crate::inventory::inventory::Inventory;
use crate::inventory::hosts::Host;
use std::collections::HashMap;
use std::sync::{Arc,Mutex,RwLock};
use crate::playbooks::traversal::RunState;

pub struct TaskHandle {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>,
}

impl TaskHandle {

    pub fn new(run_state_handle: Arc<RunState>,connection_handle: Arc<Mutex<dyn Connection>>,host_handle: Arc<RwLock<Host>>) -> Self {
        Self {
            run_state: run_state_handle,
            connection: connection_handle,
            host: host_handle,
        }
    }

    // ================================================================================
    // CONNECTION INTERACTION

    // FIXME: things like running commands go here, details are TBD.

    pub fn run(&mut self, request: &Arc<TaskRequest>, command: Arc<Command>) -> Result<(), String> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        // FIXME: use the connection to run the command
        // TODO: FIXME: think about how we want to work with command results
        // FIXME: push commands history to host?
        return Err(format!("not implemented"));
    }

    // ================================================================================
    // PLAYBOOK INTERACTION: simplified interactions with the visitor object
    // to make module code nicer.

    pub fn debug(&self, _request: &Arc<TaskRequest>, message: String) {
        self.run_state.visitor.read().unwrap().debug(message);
    }

    // ================================================================================
    // RETURN WRAPPERS FOR EVERY TASK REQUEST TYPE

    pub fn is_failed(&self, request: &Arc<TaskRequest>,  msg: String) -> Arc<TaskResponse> {
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::Failed, 
            changes: Arc::new(None),
            msg: Some(msg.clone()) 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }

    pub fn is_validated(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_validated response can only be returned for a validation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsValidated, 
            changes: Arc::new(None), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }
    
    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Create, "is_created response can only be returned for a creation request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsCreated, 
            changes: Arc::new(None), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Remove, "is_removed response can only be returned for a remove request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsRemoved, 
            changes: Arc::new(None), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Modify, "is_modified response can only be returned for a modification request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::IsModified, 
            changes: Arc::clone(&changes), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_creation response can only be returned for a query request");

        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsCreation, 
            changes: Arc::new(None), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: Arc<Option<HashMap<String,String>>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: Arc::clone(&changes), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_removal response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: Arc::new(None), 
            msg: None 
        });
        self.host.write().unwrap().record_task_response(&Arc::clone(request), &response);
        return response;
    }


}