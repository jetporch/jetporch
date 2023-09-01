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

use serde::{Deserialize};
use serde_yaml::{Value};
use std::collections::HashMap;
use crate::registry::list::Task;

#[derive(Debug,Deserialize)]
#[serde(untagged)]
pub enum AsInteger {
    String(String),
    Integer(usize),
}

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Play {
    pub name : String,
    pub groups : Vec<String>,
    pub roles : Option<Vec<Role>>,
    pub defaults: Option<serde_yaml::Mapping>,
    pub vars : Option<serde_yaml::Mapping>,
    pub vars_files: Option<Vec<String>>,
    pub ssh_user : Option<String>,
    pub ssh_port : Option<i64>,
    pub tasks : Option<Vec<Task>>,
    pub handlers : Option<Vec<Task>>,
    pub batch_size : Option<usize>,
}

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub name: String,
    pub params: Option<HashMap<String,Value>>
}


// for Tasks definitions please, see modules/list.rs
// which includes items form module_library/*.rs
