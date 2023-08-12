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
use crate::playbooks::language::AsInteger;
use crate::module_base::list::{IsTask};
use crate::module_base::list::TaskProperties;

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
    fn get_name(&self) -> String { 
        match &self.name { 
            Some(x) => x.clone(), 
            _ => String::from("") 
        } 
    }
    fn get_when(&self) -> String { 
        match &self.when { 
            Some(x) => x.clone(), 
            _ => String::from("") 
       } 
    } 
    fn get_changed_when(&self) -> String  { match &self.changed_when { Some(x) => x.clone(), _ => String::from("") }}
    fn get_retry(&self) -> String         { match &self.retry { Some(x) => x.clone(), _ => String::from("") }} 
    fn get_delay(&self) -> String         { match &self.delay { Some(x) => x.clone(), _ => String::from("") }}
    fn get_register(&self) -> String      { match &self.register { Some(x) => x.clone(), _ => String::from("") }}
}

impl IsTask for Echo {
}
