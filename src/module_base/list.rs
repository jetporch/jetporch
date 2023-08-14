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
// ABOUT: list.rs
// this is the list of all module types and works by:
// * adding the module to a list of deserializable YAML structs in task lists
// * allowing querying of the properties of the (non-polymorphic) task objects
// * allowing the module to respond to calls for task dispatch
//
// there is a small amount of boilerplate here, we have made a decision to avoid
// macros as auto-generating symbols tends to degrade some compiler abilities. see
// the online documentation for how to add modules in greater detail.
// ===================================================================================

use serde::Deserialize;
use crate::module_base::common::*;
use crate::runner::task_handle::TaskHandle;
use std::sync::Arc;

// ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (1)
use crate::module_library::echo::Echo;

#[derive(Deserialize,Debug)]
#[serde(rename_all="lowercase")]
pub enum Task {
    // ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (2)
    Echo(Echo),
}

impl Task {

    pub fn get_property(&self, property: TaskProperty) -> String { 
        return match self {
            // ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (3) 
            Task::Echo(x) => x.get_property(property), 
            _ => { panic!("module properties not registered"); },
        };
    }

    // FIXME: dispatch($self, mode: TASK_ACTION) -> Result<(), String>
    pub fn dispatch(&self, handle: Arc<TaskHandle>, request: Arc<TaskRequest>) -> TaskResponse {
        return match self {
            // ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (4) 
            Task::Echo(x) => x.dispatch(handle, request), 
            _ => { panic!("module dispatch not registered"); },
        };
    }

}




