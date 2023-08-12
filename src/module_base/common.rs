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


use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use std::collections::HashMap;

pub enum TaskProperty {
    ChangedWhen,
    Delay,
    Name,
    Register,
    Retry,
    When,
}

pub enum TaskRequestType {
    Validate,
    Query,
    Create,
    Remove,
    Modify,
}

pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Option<HashMap<String, String>>
}

pub enum TaskStatus {
    Validated,
    NeedsCreation,
    NeedsRemoval,
    NeedsModification,
    Done,
    Failed
}

pub struct TaskResponse {
    pub is: TaskStatus,
    pub changes: Option<HashMap<String, String>>
    pub msg: Option(String),
}

pub trait IsTask { 
    fn get_property(&self, property: TaskProperty) -> String;

    fn dispatch(&self, 
        context: &PlaybookContext, 
        visitor: &dyn PlaybookVisitor, 
        connection: &dyn Connection, 
        request: TaskRequest) -> TaskResponse;
}

pub fn get_optional_string_property(property: &Option<String>) -> String {
    return match property { 
        Some(x) => x.clone(), 
        _ => String::from("") 
    }
}

// ==============================================================================================
// RETURN HELPERS FOR MODULE DISPATCH METHODS
// ==============================================================================================

pub fn is_validated() -> TaskResponse {
    return TaskResponse { is: TaskStatus::Validated, changes: None, msg: None };
}

pub fn needs_creation() -> TaskResponse {
    return TaskRespones { is: TaskStatus::NeedsCreation, changes: None, msg: None };
}

pub fn needs_modification(changes: &HashMap<String,String>) -> TaskResponse {
    return TaskResponse { is: TaskStatus::NeedsModification, changes: Some(changes) };
}

pub fn needs_removal() -> TaskResponse {
    return TaskResponse { is: TaskStatus::NeedsRemoval, changes: None, msg: None };
}

pub fn failed(msg: String) -> TaskResponse() {
    return TaskResponse { is: TaskStatus::Failed, changes: None, msg: msg.clone() };
}

pub fn is_created() -> TaskResponse {
    return TaskResponse { is: TaskStatus::Created, changes: None, msg: None };
}

pub fn is_removed() -> TaskResponse {
    return TaskResponse { is: TaskStatus::Removed, changes: None, msg: None };
}

pub fn is_modified(changes: &HashMap<String,String>) -> TaskResponse() {
    return TaskResponse { is: TaskStatus::IsModified: changes: Some(changes) };
}