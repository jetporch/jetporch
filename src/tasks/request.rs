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

use std::collections::HashMap;
use std::sync::Arc;

#[derive(PartialEq)]
pub enum TaskRequestType {
    Validate,
    Query,
    Create,
    Remove,
    Modify,
    Execute,
    Passive,
}

pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Arc<Option<HashMap<String, String>>>
}

// most of the various methods in task requests are constructors for different TaskRequest type variants
// as used by task_fsm.rs. 

impl TaskRequest {
    pub fn validate() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Validate, 
                changes: Arc::new(None)
            }
        )
    }
    pub fn query() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Query, 
                changes: Arc::new(None) 
            }
        )
    }
    pub fn create() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Create, 
                changes: Arc::new(None) 
            }
        )
    }
    pub fn remove() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Remove, 
                changes: Arc::new(None)
            }
        )
    }
    pub fn modify(changes: Arc<Option<HashMap<String, String>>>) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Modify, 
                changes: Arc::clone(&changes) 
            }
        )
    }
    pub fn execute() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Execute, 
                changes: Arc::new(None) 
            }
        )
    }

    pub fn passive() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Passive, 
                changes: Arc::new(None) 
            }
        )
    }

    pub fn get_requested_changes(&self) -> Arc<Option<HashMap<String, String>>>  {
        assert!(self.request_type == TaskRequestType::Modify, "accessing change request parameters outside of TaskRequestType::Modify");
        return Arc::clone(&self.changes);
    }
}