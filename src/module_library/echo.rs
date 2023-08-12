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

#[allow(unused_imports)]
use serde::{Deserialize};
use crate::module_base::list::{IsTask,TaskProperties,get_optional_string_property};

#[derive(Deserialize,Debug)]
#[serde(tag="echo",deny_unknown_fields)]
pub struct Echo {

    // ** MODULE SPECIFIC PARAMETERS ****
    pub msg: String,

    // *** COMMON BOILERPLATE ****
    pub name: Option<String>,
    pub when: Option<String>,
    pub changed_when: Option<String>,
    pub register: Option<String>,
    pub delay: Option<String>,
    pub retry: Option<String>,
}

impl TaskProperties for Echo {

    // *** COMMON BOILERPLATE ****
    fn get_name(&self) -> String          { get_optional_string_property(&self.name) }
    fn get_when(&self) -> String          { get_optional_string_property(&self.when) } 
    fn get_changed_when(&self) -> String  { get_optional_string_property(&self.changed_when) }
    fn get_retry(&self) -> String         { get_optional_string_property(&self.retry) } 
    fn get_delay(&self) -> String         { get_optional_string_property(&self.delay) }
    fn get_register(&self) -> String      { get_optional_string_property(&self.register) }
}

impl IsTask for Echo {
}
