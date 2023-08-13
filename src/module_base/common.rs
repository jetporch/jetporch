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
// ABOUT: common.rs
// common types and interfaces (mostly) for implementing modules.  A module will
// not need to import most of these because it interacts a lot through 
// runner::task_handle which abstracts away some of these details
// ===================================================================================

//use crate::playbooks::context::PlaybookContext;
//use crate::playbooks::visitor::PlaybookVisitor;
use std::collections::HashMap;
//use crate::connection::connection::Connection;
use crate::runner::task_handle::TaskHandle;
use std::sync::Arc;

//==========================================================
// dispatch

pub enum TaskProperty {
    ChangedWhen,
    Delay,
    Name,
    Register,
    Retry,
    When,
}

//==========================================================
// request

#[derive(PartialEq)]
pub enum TaskRequestType {
    Validate,
    Query,
    Create,
    Remove,
    Modify,
}

pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Arc<HashMap<String, String>>>
}

impl TaskRequest {
    pub fn validate() -> Self {
        Self { request_type: TaskRequestType::Validate, changes: Arc::new(HashMap::new()) }
    }
    pub fn query() -> Self {
        Self { request_type: TaskRequestType::Query, changes: Arc::new(HashMap::new() }
    }
    pub fn create() -> Self {
        Self { request_type: TaskRequestType::Create, changes: Arc::new(HashMap::new() }
    }
    pub fn remove() -> Self {
        Self { request_type: TaskRequestType::Remove, changes: Arc::new(HashMap::new()) }
    }
    pub fn modify(changes: Arc<HashMap<String, String>>) -> Self {
        Self { request_type: TaskRequestType::Modify, changes: Arc::clone(changes) }
    }
}

//==========================================================
// response

#[derive(PartialEq)]
pub enum TaskStatus {
    IsCreated,
    IsRemoved,
    IsModified,
    IsValidated,
    IsChanged,
    NeedsCreation,
    NeedsRemoval,
    NeedsModification,
    Failed
}

pub struct TaskResponse {
    pub is: TaskStatus,
    pub changes: Arc<HashMap<String, String>>,
    pub msg: Option<String>,
}

//==========================================================
// interfaces & helper functions

pub trait IsTask { 
    fn get_property(&self, property: TaskProperty) -> String;

    fn dispatch(&self, 
        handle: &TaskHandle, 
        request: TaskRequest) -> TaskResponse;
}

pub fn get_property(property: &Option<String>) -> String {
    return match property { 
        Some(x) => x.clone(), 
        _ => String::from("") 
    }
}
