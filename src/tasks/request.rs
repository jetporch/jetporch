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

#[derive(PartialEq)]
pub enum TaskRequestType {
    Validate,
    Query,
    Create,
    Remove,
    Modify,
}

pub struct TaskRequest {
    pub task: Arc<Task>,
    pub request_type: TaskRequestType,
    pub changes: Arc<HashMap<String, String>>
}

// most of the various methods in task requests are constructors for different TaskRequest type variants
// as used by task_fsm.rs. 

impl TaskRequest {
    pub fn validate(task: Arc<Task>) -> Arc<Self> {
        return Arc::new(
            Self { 
                task: Arc::clone(&task), 
                request_type: TaskRequestType::Validate, 
                changes: Arc::new(HashMap::new()) 
            }
        )
    }
    pub fn query(task: Arc<Task>) -> Arc<Self> {
        return Arc::new(
            Self { 
                task: Arc::clone(&task), 
                request_type: TaskRequestType::Query, 
                changes: Arc::new(HashMap::new()) 
            }
        )
    }
    pub fn create(task: Arc<Task>) -> Arc<Self> {
        return Arc::new(
            Self { 
                task: Arc::clone(&task), 
                request_type: TaskRequestType::Create, 
                changes: Arc::new(HashMap::new()) 
            }
        )
    }
    pub fn remove(task: Arc<Task>) -> Arc<Self> {
        return Arc::new(
            Self { 
                task: Arc::clone(&task), 
                request_type: TaskRequestType::Remove, 
                changes: Arc::new(HashMap::new()) 
            }
        )
    }
    pub fn modify(task: Arc<Task>, changes: Arc<HashMap<String, String>>) -> Arc<Self> {
        return Arc::new(
            Self { 
                task: Arc::clone(&task), 
                request_type: TaskRequestType::Modify, 
                changes: Arc::clone(&changes) 
            }
        )
    }

    pub fn get_requested_changes(&self) {
        assert!(self.request_type == TaskRequestType::Modify, "accessing change request parameters outside of TaskRequestType::Modify");
        return Arc::clone(&changes);
    }
}