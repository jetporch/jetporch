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

/*
#[allow(unused_imports)]
use serde::{Deserialize};
use crate::playbooks::language::AsInteger;
use crate::module_base::list::{IsTask};
use crate::module_base::list::TaskProperties;

#[derive(Deserialize,Debug)]
#[serde(tag="shell",deny_unknown_fields)]
pub struct Shell {

    // ** MODULE SPECIFIC PARAMETERS ****
    pub cmd: String,

    // *** COMMON BOILERPLATE ****
    pub name: String,
    pub when: Option<String>,
    pub changed_when: Option<String>,
    pub register: Option<String>,
    pub delay: Option<AsInteger>,
    pub retry: Option<AsInteger>,
}

impl TaskProperties for Shell {
    fn get_name(&self) -> String       { return self.name.or_else("");                   }
    fn get_when(&self) -> String       { return self.when.or_else("".to_string())        } 
    fn get_changed_when(&self) String  { return self.changed_when.or_else("".to_string() }
    fn get_retry(&self) -> usize       { return self.retry.or_else(0usize);              }
    fn get_delay(&self) -> usize       { return self.delay_or_else(0usize);              }
    fn get_register(&self) -> <String> { return self.register.or_else("".to_string());   }
}

impl IsTask for Shell {

}

*/