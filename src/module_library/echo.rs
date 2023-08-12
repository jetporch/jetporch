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

//#[allow(unused_imports)]
use serde::{Deserialize};
use crate::module_base::common::*;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;

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
    pub when: Option<String>,
}

impl IsTask for Echo {

    /** COMMON MODULE BOILERPLATE **/
    fn get_property(&self, property: TaskProperty) -> String { 
        return match property {
            TaskProperty::ChangedWhen => get_optional_string_property(&self.changed_when),
            TaskProperty::Delay => get_optional_string_property(&self.delay),
            TaskProperty::Register => get_optional_string_property(&self.register),
            TaskProperty::Retry => get_optional_string_property(&self.retry),
            TaskProperty::Name => get_optional_string_property(&self.name),
            TaskProperty::When => get_optional_string_property(&self.when), 
        }
    }

    /** MODULE SPECIFIC IMPLEMENTATION **/
    fn dispatch(&self, 
        context: &PlaybookContext, 
        visitor: &dyn PlaybookVisitor, 
        connection: &dyn Connection, 
        request: TaskRequest) -> TaskResponse {
    
        match request.request_type {

            TaskRequestType::Validate => {
                // the echo module has nothing to validate
                return is_validated();
            },
    
            TaskRequestType::Query => {
                // can also return a hashmap of changes in request.changes we could conditionally consider 
                return needs_creation();
            },
    
            TaskRequestType::Create => {
                context.debug(self.msg);
                return done();
            },
    
            TaskRequestType::Remove => {
                panic!("impossible");
            },

            TaskRequestType::Modify => {
                panic!("impossible");
            },
    

    
        }
    }
}
