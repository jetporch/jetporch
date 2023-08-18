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

use crate::tasks::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use std::sync::Arc;
use crate::tasks::response::TaskResponse;

//==========================================================
// Common parameters for all tasks

pub enum TaskProperty {
    ChangedWhen,
    Delay,
    Name,
    Register,
    Retry,
    When,
}


//==========================================================
// Methods we can call on a task

pub trait IsTask { 
    fn get_property(&self, property: TaskProperty) -> String;
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Arc<TaskResponse>;
}

pub fn get_property_or_default(property: &Option<String>, default: &String) -> String {
    return match property { 
        Some(x) => x.clone(), 
        _ => default.clone()
    }
}

pub fn get_property(property: &Option<String>) -> String {
    return match property { 
        Some(x) => x.clone(), 
        _ => String::from("") 
    }
}