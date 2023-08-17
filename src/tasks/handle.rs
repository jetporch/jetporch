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

use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::connection::connection::Connection;
use crate::tasks::request::{TaskRequest, TaskRequestType}
use crate::tasks::response::{TaskResponse, TaskStatus};
use crate::connection::command::Command;
use crate::inventory::inventory::Inventory;
use std::collections::HashMap;
use std::sync::{Arc,Mutex,RwLock};

pub struct TaskHandle {
    inventory: Arc<RwLock<Inventory>>, 
    context: Arc<RwLock<PlaybookContext>>,
    visitor: Arc<Mutex<dyn PlaybookVisitor>>, 
    connection: Arc<RwLock<dyn Connection>>,
    host: Arc<RwLock<Host>>,
}

impl TaskHandle {

    pub fn new(
        inventory_handle: &Arc<RwLock<Inventory>>, 
        context_handle: &Arc<Mutex<PlaybookContext>>, 
        visitor_handle: &Arc<RwLock<dyn PlaybookVisitor>>, 
        connection_handle: &Arc<RwLock<dyn Connection>>,
        host_handle: &Arc<RwLock<Host>>) -> Self {

        Self {
            inventory: Arc::clone(inventory_handle),
            context: Arc::clone(context_handle),
            visitor: Arc::clone(visitor_handle),
            connection: Arc::clone(connection_handle),
            host: Arc::clone(host_handle),
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
    }

    // ================================================================================
    // PLAYBOOK INTERACTION: simplified interactions with the visitor object
    // to make module code nicer.

    pub fn debug(&self, _request: &Arc<TaskRequest>, message: String) {
        self.visitor.lock().unwrap().debug(message);
    }

    // ================================================================================
    // RETURN WRAPPERS FOR EVERY TASK REQUEST TYPE

    pub fn is_failed(&self, request: &Arc<TaskRequest>,  msg: String) -> Arc<TaskResponse> {
        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::Failed, 
            changes: Arc::new(HashMap::new()), 
            msg: Some(msg.clone()) 
        });
        self.host.write().record_task_response(Arc::clone(&response));
        return response;
    }

    pub fn is_validated(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Validate, "is_validated response can only be returned for a validation request");
        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::IsValidated, 
            changes: Arc::new(HashMap::new()), 
            msg: None 
        });
        self.host.write().record_task_response(Arc::clone(&response));
        return response;
    }
    
    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Create, "is_created response can only be returned for a creation request");
        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::IsCreated, 
            changes: Arc::new(HashMap::new()), 
            msg: None 
        });
        self.host.write().record_task_response(Arc::clone(&response));
        return response;
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Remove, "is_removed response can only be returned for a remove request");
        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::IsRemoved, 
            changes: Arc::new(HashMap::new()), 
            msg: None 
        });
        self.host.write().record_task_response(Arc::clone(&response));
        return response;
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: Arc<HashMap<String,String>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Modify, "is_modified response can only be returned for a modification request");
        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::IsModified, 
            changes: Arc::clone(&changes), 
            msg: None 
        });
        self.host.write().record_task_response(response);
        return response;
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Query, "needs_creation response can only be returned for a query request");

        let response = Arc::new(TaskResponse { 
            request: Arc::clone(request), 
            status: TaskStatus::NeedsCreation, 
            changes: Arc::new(HashMap::new()), 
            msg: None 
        });
        self.host.write().record_task_response(response);
        return response;
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: Arc<HashMap<String,String>>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Query, "needs_modification response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: Arc::clone(&changes), 
            msg: None 
        });
        self.host.write().record_task_response(response);
        return response;
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequest::Query, "needs_removal response can only be returned for a query request");
        let response = Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: Arc::new(HashMap::new()), 
            msg: None 
        });
        self.host.write().record_task_response(response);
        return response;
    }


}