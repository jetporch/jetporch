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

use crate::module_base::common::{TaskRequest,TaskRequestType,TaskResponse,TaskProperty,get_property};
use crate::runner::task_handle::TaskHandle;
use crate::module_base::common::IsTask;
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;

#[derive(Deserialize,Debug)]
#[serde(tag="echo",deny_unknown_fields)]
pub struct Echo {

    // ** MODULE SPECIFIC PARAMETERS ****
    pub msg: String,

    // *** COMMON MODULE BOILERPLATE ****
    pub changed_when: Option<String>,
    pub delay: Option<String>,
    pub name: Option<String>,
    pub register: Option<String>,
    pub retry: Option<String>,
    pub when: Option<String>
}

impl IsTask for Echo {

    /** COMMON MODULE BOILERPLATE **/
    fn get_property(&self, property: TaskProperty) -> String { 
        return match property {
            TaskProperty::ChangedWhen => get_property(&self.changed_when),
            TaskProperty::Delay => get_property(&self.delay),
            TaskProperty::Register => get_property(&self.register),
            TaskProperty::Retry => get_property(&self.retry),
            TaskProperty::Name => get_property(&self.name),
            TaskProperty::When => get_property(&self.when), 
        }
    }

    /** MODULE SPECIFIC IMPLEMENTATION **/
    fn dispatch(&self, handle: Arc<TaskHandle>, request: Arc<TaskRequest>) -> TaskResponse {
    
        match request.request_type {

            TaskRequestType::Validate => {
                return handle.is_validated(&request);
            },

            TaskRequestType::Query => {
                return handle.needs_creation(&request)
            },
    
            TaskRequestType::Create => {
                handle.debug(&request, self.msg.clone());
                return handle.is_created(&request)
            },
    
            TaskRequestType::Remove => {
                panic!("this module does not remove resources");
            },

            TaskRequestType::Modify => {
                panic!("this module does not modify resources");
            },
            _ => { panic!("invalid request type") }
    
        }
    }

}
