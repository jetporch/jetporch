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

// ADD NEW MODULES HERE, KEEP ALPHABETIZED BY SECTION

// commands
use crate::modules::commands::shell::ShellTask;

// control
use crate::modules::control::assert::AssertTask;
use crate::modules::control::echo::EchoTask;
use crate::modules::control::fail::FailTask;
use crate::modules::control::facts::FactsTask;
use crate::modules::control::set::SetTask;

// files
use crate::modules::files::copy::CopyTask;
use crate::modules::files::directory::DirectoryTask;
use crate::modules::files::file::FileTask;
use crate::modules::files::template::TemplateTask;

// packages
use crate::modules::packages::apt::AptTask;
use crate::modules::packages::dnf::DnfTask;

// services
use crate::modules::services::sd_service::SystemdServiceTask;

#[allow(non_camel_case_types)]
#[derive(Deserialize,Debug)]
#[serde(rename_all="lowercase")]
pub enum Task {
    // ADD NEW MODULES HERE, KEEP ALPHABETIZED BY NAME
    Apt(AptTask),
    Assert(AssertTask),
    Copy(CopyTask),
    Dnf(DnfTask),
    Directory(DirectoryTask),
    Echo(EchoTask),
    Fail(FailTask),
    Facts(FactsTask),
    File(FileTask),
    Sd_Service(SystemdServiceTask),
    Set(SetTask),
    Shell(ShellTask),
    Template(TemplateTask),
}

impl Task {

    pub fn get_module(&self) -> String {
        return match self {
            Task::Apt(x)        => x.get_module(),
            Task::Assert(x)     => x.get_module(),
            Task::Copy(x)       => x.get_module(),
            Task::Dnf(x)        => x.get_module(),
            Task::Directory(x)  => x.get_module(),
            Task::Echo(x)       => x.get_module(),
            Task::Facts(x)      => x.get_module(), 
            Task::Fail(x)       => x.get_module(), 
            Task::File(x)       => x.get_module(), 
            Task::Sd_Service(x) => x.get_module(),
            Task::Set(x)        => x.get_module(), 
            Task::Shell(x)      => x.get_module(), 
            Task::Template(x)   => x.get_module(), 
        };
    }

    pub fn get_name(&self) -> Option<String> {
        return match self {
            Task::Apt(x)        => x.get_name(),
            Task::Assert(x)     => x.get_name(),
            Task::Copy(x)       => x.get_name(), 
            Task::Dnf(x)        => x.get_name(),
            Task::Directory(x)  => x.get_name(),
            Task::Echo(x)       => x.get_name(),
            Task::Facts(x)      => x.get_name(),
            Task::Fail(x)       => x.get_name(), 
            Task::File(x)       => x.get_name(), 
            Task::Sd_Service(x) => x.get_name(),
            Task::Set(x)        => x.get_name(),
            Task::Shell(x)      => x.get_name(), 
            Task::Template(x)   => x.get_name(), 
        };
    }

    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        // ADD NEW MODULES HERE, KEEP ALPHABETIZED BY NAME
        return match self {
            Task::Apt(x)        => x.evaluate(handle, request),
            Task::Assert(x)     => x.evaluate(handle, request),
            Task::Copy(x)       => x.evaluate(handle, request), 
            Task::Dnf(x)        => x.evaluate(handle, request),
            Task::Directory(x)  => x.evaluate(handle, request), 
            Task::Echo(x)       => x.evaluate(handle, request),
            Task::Fail(x)       => x.evaluate(handle, request),  
            Task::Facts(x)      => x.evaluate(handle, request),
            Task::File(x)       => x.evaluate(handle, request), 
            Task::Sd_Service(x) => x.evaluate(handle, request),
            Task::Set(x)        => x.evaluate(handle, request),
            Task::Shell(x)      => x.evaluate(handle, request), 
            Task::Template(x)   => x.evaluate(handle, request), 
        };
    }

    // ==== END MODULE REGISTRY CONFIG ====

    pub fn get_display_name(&self) -> String {
        return match self.get_name() { Some(x) => x, _ => self.get_module()  }
    }

}




