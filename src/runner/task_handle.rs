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

import crate::playbooks::context::PlaybookContext;
import crate::playbooks::visitor::PlaybookVisitor;
import crate::connection::Connection;
import crate::module_base::common::{TaskRequestType, TaskResponse};
import crate::connection::command::Command;

pub struct TaskHandle {
    context: &PlaybookContext,
    visitor: &dyn PlaybookVisitor, 
    connection: &dyn Connection,
    request_type: TaskRequestType,
    changes: Vec<String>
    commands: Vec<Command>
}

impl TaskHandle {

    pub new() -> Self {
        Self {
            context: context,
            visitor: visitor,
            connection: connection,
            request_type: request_type,
            pub changes: Vec::new(),
            pub commands: Vec::new(),
        }
    }

    // ================================================================================
    // CHANGE MANAGEMENT

    pub fn suggest_change(change: String) {
        assert(request_type == TaskRequestType::Query, "changes can only be suggested in query stage");
        self.changes.push(change.clone());
    }

    // ================================================================================
    // CONNECTION INTERACTION

    // FIXME: things like running commands go here, details are TBD.

    pub fn run(command: Command) {
        assert!(self.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        commands.push(command);
    }

    // ================================================================================
    // PLAYBOOK INTERACTION

    pub fn debug(message: String) {
        context.debug(message);
    }

    // ================================================================================
    // RETURN WRAPPERS FOR ANY TASK REQUEST TYPE

    pub fn is_failed(msg: String) -> TaskResponse() {
        return TaskResponse { is: TaskStatus::Failed, changes: None, msg: msg.clone() };
    }

    // ================================================================================
    // RETURN WRAPPERS FOR TASK DISPATCH (VALIDATION)

    pub fn is_validated() -> TaskResponse {
        assert!(self.request_type == TaskRequestType::Validate, "module impl error. unexpected return type, not a validation mode");
        assert!(self.changes.is_empty());
        return TaskResponse { is: TaskStatus::Validated, changes: None, msg: None };
    }

    // ================================================================================
    // RETURN WRAPPERS FOR TASK DISPATCH (ACTION MODES)
    
    pub fn is_created() -> TaskResponse {
        assert!(self.request_type == TaskRequestType::Create, "module impl error. unexpected return type, not a creation mode");
        return TaskResponse { is: TaskStatus::Created, changes: None, msg: None };
    }
    
    pub fn is_removed() -> TaskResponse {
        assert!(self.request_type == TaskRequestType::Remove, "module impl error. unexpected return type, not a removal mode");
        return TaskResponse { is: TaskStatus::Removed, changes: None, msg: None };
    }
    
    pub fn is_modified(changes: &HashMap<String,String>) -> TaskResponse() {
        assert!(self.request_type == TaskRequestType::Modify, "module impl error. unexpected return type, not a modification mode");
        assert!(!self.changes.is_empty(), "module impl error. returning a modification with no fields changed, return failed instead");
        return TaskResponse { is: TaskStatus::IsModified: changes: Some(changes) };
    }

    // ================================================================================
    // RETURN WRAPPERS FOR TASK DISPATCH (QUERY)

    pub fn needs_creation() -> TaskResponse {
        assert!(self.request_type == TaskRequestType::Query, "module impl error. unexpected return type, not a query mode");
        assert!(self.changes.is_empty());
        return TaskRespones { is: TaskStatus::NeedsCreation, changes: None, msg: None };
    }
    
    pub fn needs_modification(changes: &HashMap<String,String>) -> TaskResponse {
        assert!(self.request_type == TaskRequestType::Query, "module impl error. unexpected return type, not a query mode");
        assert!(!self.changes.is_empty(), "module impl error. requesting a modification with no changes");
        return TaskResponse { is: TaskStatus::NeedsModification, changes: Some(changes) };
    }
    
    pub fn needs_removal() -> TaskResponse {
        assert!(self.changes.is_empty(), "module impl error. requesting a removal with changes flagged");
        return TaskResponse { is: TaskStatus::NeedsRemoval, changes: None, msg: None };
    }


}