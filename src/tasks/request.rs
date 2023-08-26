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
use crate::tasks::fields::Field;
use std::collections::HashSet;

// task requests are objects given to modules (and the task FSM) that
// describe what questions we are asking of them. In the case of 
// modifications, this includes the list (map) of parameters to change
// as returned by the query request

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
    pub changes: HashSet<Field>
}

// most of the various methods in task requests are constructors for different TaskRequest type variants
// as used by task_fsm.rs. 

impl TaskRequest {

    #[inline]
    pub fn validate() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Validate, 
                changes: HashSet::new()
            }
        )
    }

    #[inline]
    pub fn query() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Query, 
                changes: HashSet::new() 
            }
        )
    }

    #[inline]
    pub fn create() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Create, 
                changes: HashSet::new()
            }
        )
    }

    #[inline]
    pub fn remove() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Remove, 
                changes: HashSet::new()
            }
        )
    }

    #[inline]
    pub fn modify(changes: HashSet<Field>) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Modify, 
                changes: changes
            }
        )
    }

    #[inline]
    pub fn execute() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Execute, 
                changes: HashSet::new() 
            }
        )
    }

    #[inline]
    pub fn passive() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Passive, 
                changes: HashSet::new() 
            }
        )
    }

    // FIXME, hashset?
    //pub fn get_requested_changes(&self) -> Arc<Option<HashMap<String, String>>>  {
    //    assert!(self.request_type == TaskRequestType::Modify, "accessing change request parameters outside of TaskRequestType::Modify");
    //    return Arc::clone(&self.changes);
    //}
}