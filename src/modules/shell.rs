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

use crate::tasks::common::{TaskProperty,IsTask,get_property,get_property_or_default};
use crate::tasks::handle::TaskHandle;
use crate::tasks::response::TaskResponse;
use crate::tasks::request::{TaskRequestType,TaskRequest};
use crate::connection::command::cmd_info;
use std::sync::Arc;
//#[allow(unused_imports)]
use serde::{Deserialize};

// =======================================================================
// MODULE STRUCTURE
// =======================================================================

static MODULE_NAME : &'static str = "Shell";

#[derive(Deserialize,Debug)]
#[serde(tag="shell",deny_unknown_fields)]
pub struct Shell {

    // ** MODULE SPECIFIC PARAMETERS ****
    pub cmd: String,
    pub verbose: Option<bool>,

    // *** COMMON MODULE BOILERPLATE ****
    pub changed_when: Option<String>,
    pub delay: Option<String>,
    pub name: Option<String>,
    pub register: Option<String>,
    pub retry: Option<String>,
    pub when: Option<String>
}

impl IsTask for Shell {

    // =======================================================================
    // FIELD ACCESS BOILERPLATE
    // =======================================================================

    fn get_property(&self, property: TaskProperty) -> String { 
        return match property {
            TaskProperty::ChangedWhen => get_property(&self.changed_when),
            TaskProperty::Delay => get_property(&self.delay),
            TaskProperty::Register => get_property(&self.register),
            TaskProperty::Retry => get_property(&self.retry),
            TaskProperty::Name => get_property_or_default(&self.name, &String::from(MODULE_NAME)),
            TaskProperty::When => get_property(&self.when), 
        }
    }

    // =======================================================================
    // MODULE SPECIFIC IMPLEMENTATION
    // =======================================================================

    /** MODULE SPECIFIC IMPLEMENTATION **/
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Validate => {
                return Ok(handle.is_validated(&request));
            },

            TaskRequestType::Query => {
                return Ok(handle.needs_execution(&request));
            },

            /*
            TaskRequestType::Create => {
                panic!("this module does not create resources");
            },
            */
    
            TaskRequestType::Execute => {
                let result = handle.run(&request, &self.cmd.clone());
                let (rc, out) = cmd_info(&result);

                if self.verbose.is_some() && self.verbose.unwrap() {
                    let display = vec![ format!("rc: {}",rc), format!("out: {}", out) ];
                    handle.debug_lines(&request, &display);
                }

                return result;
            },
    
            /*
            TaskRequestType::Remove => {
                panic!("this module does not remove resources");
            },

            TaskRequestType::Modify => {
                panic!("this module does not modify resources");
            },
            */
            
            _ => { panic!("invalid request type") }
    
        }
    }

}
