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

use serde::Deserialize;
use crate::tasks::*;
use std::sync::Arc;

// note: there is some repetition in this module that we would rather not have
// however, it comes from a conflict between polymorphic dispatch macros + traits
// and a lack of data-inheritance in structs. please ignore it the best you can 
// and this may be improved later.

// ADD NEW MODULES HERE, KEEP ALPHABETIZED
use crate::modules::echo::Echo;
use crate::modules::shell::Shell;

#[derive(Deserialize,Debug)]
#[serde(rename_all="lowercase")]
pub enum Task {
    // ADD NEW MODULES HERE, KEEP ALPHABETIZED
    Echo(Echo),
    Shell(Shell),
}

impl Task {

    pub fn get_module(&self) -> String {
        return match self {
            // ADD NEW MODULES HERE, KEEP ALPHABETIZED
            Task::Echo(x) => x.get_module(), 
            Task::Shell(x) => x.get_module(), 
        };
    }

    pub fn get_name(&self) -> Option<String> {
        return match self {
            // ADD NEW MODULES HERE, KEEP ALPHABETIZED
            Task::Echo(x) => x.get_name(), 
            Task::Shell(x) => x.get_name(), 
        };
    }

    pub fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
        // ADD NEW MODULES HERE, KEEP ALPHABETIZED
        return match self {
            Task::Echo(x)  => x.dispatch(handle, request), 
            Task::Shell(x) => x.dispatch(handle, request), 
        };
    }

    pub fn get_display_name(&self) -> String {
        return match self.get_name() {
            Some(x) => x,
            _ => self.get_module()
        }
    }

}




