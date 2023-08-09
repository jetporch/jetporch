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

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Play {
    jet : JetHeader,
    groups: Vec<String>,
    roles : Option<Vec<String>>,
    force_vars: Option<HashMap<String,Value>>,
    defaults: Option<HashMap<String,Value>>,
    remote_user: Option<String>,
    tasks: Option<Vec<Task>>,
    handlers: Option<Vec<Task>>
}

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JetHeader {
    version: String
}

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Task {
    Include(Include),
    Shell(Shell),
    External(External),
}

// ====

pub trait TaskProperties {
    fn get_when(&self) -> Option<String>;
    fn get_changed_when(&self) -> Option<String>;
    fn get_retry(&self) -> Option<String>;
    fn get_delay(&self) -> Option<String>;
}

pub trait IsTask: TaskProperties { // + Runnable?
    fn run(&self) -> Result<(), String>;
}

#[macro_export]
macro_rules! define_task {
    ($name:ident { $($fname:ident : $ftype:ty),* }) => {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            pub name: Option<String>,
            pub when: Option<String>,
            pub changed_when: Option<String>,
            pub register: Option<String>,
            pub delay: Option<String>,
            pub retry: Option<String>,
            $(pub $fname : $ftype),*
        }
    };
}

// ======

// helper macro to avoid repetition of "basic" impl Coordinates
#[macro_export]
macro_rules! add_task_properties { 
    ($T:ident) => {
        impl TaskProperties for $T {
            fn get_when(&self) -> Option<String> { return self.when } 
            fn get_changed_when(&self) -> Option<String> { return self.changed_when }
            fn get_retry(&self) -> Option<String> { return self.retry }
            fn get_delay(&self) -> Option<String> { return self.delay }
        }
    }
}

// ======
// include.rs

define_task!(Include { path: String });
add_task_properties!(Include);

impl IsTask for Include {
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}

// ======
// shell.rs

define_task!(Shell { cmd: String });
add_task_properties!(Shell);

impl IsTask for Shell {
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}

// ======
// external.rs

define_task!(External { module: String, params: HashMap<String, Value> });
add_task_properties!(External);

impl IsTask for External {
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}