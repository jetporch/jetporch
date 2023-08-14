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
// and does things correctly
// ===================================================================================

use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::connection::connection::Connection;
use crate::module_base::common::{TaskRequest, TaskRequestType, TaskResponse, TaskStatus};
use crate::connection::command::Command;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub struct TaskHandle {
    context: Arc<Mutex<PlaybookContext>>,
    visitor: Arc<Mutex<dyn PlaybookVisitor>>, 
    connection: Arc<Mutex<dyn Connection>>,
    pub changes: Vec<String>,
    pub commands: Vec<Arc<Command>>,
}

impl TaskHandle {

    pub fn new(c: &Arc<Mutex<PlaybookContext>>, v: &Arc<Mutex<dyn PlaybookVisitor>>, conn: &Arc<Mutex<dyn Connection>>) -> Self {
        Self {
            context: Arc::clone(c),
            visitor: Arc::clone(v),
            connection: Arc::clone(conn),
            changes: Vec::new(),
            commands: Vec::new(),
        }
    }

    // ================================================================================
    // CHANGE MANAGEMENT

    pub fn suggest_change(&self, request: Arc<TaskRequest>, change: String) {
        assert!(request.request_type == TaskRequestType::Query, "changes can only be suggested in query stage");
        self.changes.push(change.clone());
    }

    // ================================================================================
    // CONNECTION INTERACTION

    // FIXME: things like running commands go here, details are TBD.

    pub fn run(&mut self, request: Arc<TaskRequest>, command: Arc<Command>) {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        self.commands.push(Arc::clone(&command));
    }

    // ================================================================================
    // PLAYBOOK INTERACTION

    pub fn debug(&self, _request: Arc<TaskRequest>, message: String) {
        self.visitor.lock().unwrap().debug(message);
    }

    // ================================================================================
    // RETURN WRAPPERS FOR EVERY TASK REQUEST TYPE

    pub fn is_failed(&self, _request: Arc<TaskRequest>,  msg: String) -> TaskResponse {
        return TaskResponse { is: TaskStatus::Failed, changes: Arc::new(HashMap::new()), msg: Some(msg.clone()) };
    }

    pub fn is_validated(&self, request: Arc<TaskRequest>, ) -> TaskResponse {
        return TaskResponse { is: TaskStatus::IsValidated, changes: Arc::new(HashMap::new()), msg: None };
    }
    
    pub fn is_created(&self, request: Arc<TaskRequest>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::IsCreated, changes: Arc::new(HashMap::new()), msg: None };
    }
    
    pub fn is_removed(&self, request: Arc<TaskRequest>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::IsRemoved, changes: Arc::new(HashMap::new()), msg: None };
    }
    
    pub fn is_modified(&self, request: Arc<TaskRequest>, changes: Arc<HashMap<String,String>>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::IsModified, changes: Arc::clone(&changes), msg: None };
    }

    pub fn needs_creation(&self, request: Arc<TaskRequest>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::NeedsCreation, changes: Arc::new(HashMap::new()), msg: None };
    }
    
    pub fn needs_modification(&self, request: Arc<TaskRequest>, changes: Arc<HashMap<String,String>>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::NeedsModification, changes: Arc::clone(&changes), msg: None };
    }
    
    pub fn needs_removal(&self, request: Arc<TaskRequest>) -> TaskResponse {
        return TaskResponse { is: TaskStatus::NeedsRemoval, changes: Arc::new(HashMap::new()), msg: None };
    }


}