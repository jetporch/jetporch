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
// and this may be improved later. If there was no Enum, we could have
// polymorphic dispatch, but traversal would lose a lot of serde benefits.

// ADD NEW MODULES HERE, KEEP ALPHABETIZED
use crate::modules::copy::CopyTask;
use crate::modules::directory::DirectoryTask;
use crate::modules::echo::EchoTask;
use crate::modules::file::FileTask;
use crate::modules::shell::ShellTask;
use crate::modules::template::TemplateTask;

#[derive(Deserialize,Debug)]
#[serde(rename_all="lowercase")]
pub enum Task {
    // ADD NEW MODULES HERE, KEEP ALPHABETIZED
    Copy(CopyTask),
    Directory(DirectoryTask),
    Echo(EchoTask),
    File(FileTask),
    Shell(ShellTask),
    Template(TemplateTask),
}

impl Task {

    pub fn get_module(&self) -> String {
        return match self {
            // ADD NEW MODULES HERE, KEEP ALPHABETIZED
            Task::Copy(x)      => x.get_module(), 
            Task::Directory(x) => x.get_module(),
            Task::Echo(x)      => x.get_module(), 
            Task::File(x)      => x.get_module(), 
            Task::Shell(x)     => x.get_module(), 
            Task::Template(x)  => x.get_module(), 
        };
    }

    pub fn get_name(&self) -> Option<String> {
        return match self {
            // ADD NEW MODULES HERE, KEEP ALPHABETIZED
            Task::Copy(x)      => x.get_name(), 
            Task::Directory(x) => x.get_name(),
            Task::Echo(x)      => x.get_name(), 
            Task::File(x)      => x.get_name(), 
            Task::Shell(x)     => x.get_name(), 
            Task::Template(x)  => x.get_name(), 
        };
    }

    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        // ADD NEW MODULES HERE, KEEP ALPHABETIZE
        return match self {
            Task::Copy(x)      => x.evaluate(handle, request), 
            Task::Directory(x) => x.evaluate(handle, request), 
            Task::Echo(x)      => x.evaluate(handle, request), 
            Task::File(x)      => x.evaluate(handle, request), 
            Task::Shell(x)     => x.evaluate(handle, request), 
            Task::Template(x)  => x.evaluate(handle, request), 
        };
    }

    // ==== END MODULE REGISTRY CONFIG ====

    pub fn get_display_name(&self) -> String {
        return match self.get_name() { Some(x) => x, _ => self.get_module()  }
    }

}




