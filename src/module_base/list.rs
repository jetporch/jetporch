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
use crate::module_base::common::*;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;

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
            Task::Echo(x) => x.get_proprerty(property), 
            _ => { panic!("internal error"); },
        };
    }

    // FIXME: dispatch($self, mode: TASK_ACTION) -> Result<(), String>
    fn dispatch(&self, context: &PlaybookContext, visitor: &dyn PlaybookVisitor, connection: &dyn Connection, request: TaskRequest) -> TaskResponse {
        return match self {
            // ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (3) 
            Task::Echo(x) => self.dispatch(context, visitor, connection, request), 
            _ => { panic!("internal error"); },
        };
    }

}




