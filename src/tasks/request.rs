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

//use std::collections::HashMap;
use std::sync::Arc;
use crate::tasks::fields::Field;
use std::vec::Vec;

// task requests are objects given to modules (and the task FSM) that
// describe what questions we are asking of them. In the case of 
// modifications, this includes the list (map) of parameters to change
// as returned by the query request

#[derive(Debug,PartialEq)]
pub enum TaskRequestType {
    Validate,
    Query,
    Create,
    Remove,
    Modify,
    Execute,
    Passive,
}

#[derive(Debug)]
pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Vec<Field>,
    pub sudo_details: Option<SudoDetails>
}

#[derive(Debug,PartialEq,Clone)]
pub struct SudoDetails {
    pub user: Option<String>,
    pub template: String
}

// most of the various methods in task requests are constructors for different TaskRequest type variants
// as used by task_fsm.rs. 

impl TaskRequest {

    pub fn validate() -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Validate, 
                changes: Vec::new(),
                sudo_details: None
            }
        )
    }

    pub fn query(sudo_details: SudoDetails) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Query, 
                changes: Vec::new(),
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

    pub fn create(sudo_details: SudoDetails) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Create, 
                changes: Vec::new(),
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

    pub fn remove(sudo_details: SudoDetails) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Remove, 
                changes: Vec::new(),
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

    pub fn modify(sudo_details: SudoDetails, changes: Vec<Field>) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Modify, 
                changes: changes,
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

    pub fn execute(sudo_details: SudoDetails) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Execute, 
                changes: Vec::new(),
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

    pub fn passive(sudo_details: SudoDetails) -> Arc<Self> {
        return Arc::new(
            Self { 
                request_type: TaskRequestType::Passive, 
                changes: Vec::new(),
                sudo_details: Some(sudo_details.clone())
            }
        )
    }

}