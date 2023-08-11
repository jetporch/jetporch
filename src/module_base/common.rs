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
use crate::playbooks::language::{AsInteger};

pub trait TaskProperties {
    // FIXME: add failed_when, other keywords 
    fn get_name(&self) -> String;
    fn get_when(&self) -> Option<String>;
    fn get_changed_when(&self) -> Option<String>;
    fn get_retry(&self) -> Option<AsInteger>;
    fn get_delay(&self) -> Option<AsInteger>;
    fn get_register(&self) -> Option<String>;
}

#[macro_export]
macro_rules! define_task {
    ($name:ident { $($fname:ident : $ftype:ty),* }) => {
        #[derive(Debug,Deserialize)]
        #[serde(deny_unknown_fields,tag="$name")]
        pub struct $name {
            pub name: String,
            pub when: Option<String>,
            pub changed_when: Option<String>,
            pub register: Option<String>,
            pub delay: Option<AsInteger>,
            pub retry: Option<AsInteger>,
            $(pub $fname : $ftype),*
        }
    };
}

// TODO: maybe implement clone on AsInteger?

pub(crate) use define_task; 

#[macro_export]
macro_rules! add_task_properties { 
    ($T:ident) => {
        impl TaskProperties for $T {
            fn get_name(&self) -> String { return self.name.clone() }
            fn get_when(&self) -> Option<String> { return self.when.clone() } 
            fn get_changed_when(&self) -> Option<String> { return self.changed_when.clone() }
            fn get_retry(&self) -> Option<AsInteger> { return Some(AsInteger::Integer(0)); }
            fn get_delay(&self) -> Option<AsInteger> { return Some(AsInteger::Integer(0)); }
            fn get_register(&self) -> Option<String> { return self.register.clone(); }
        }
    }
}

pub(crate) use add_task_properties; 

pub trait IsTask: TaskProperties { 
    fn get_module(&self) -> String;
    fn run(&self) -> Result<(), String>;
}

