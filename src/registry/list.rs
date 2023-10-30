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

// accessctl
use crate::modules::access::user::UserTask;

// commands
use crate::modules::commands::shell::ShellTask;

// control
use crate::modules::control::assert::AssertTask;
use crate::modules::control::debug::DebugTask;
use crate::modules::control::echo::EchoTask;
use crate::modules::control::fail::FailTask;
use crate::modules::control::facts::FactsTask;
use crate::modules::control::set::SetTask;

// files
use crate::modules::files::copy::CopyTask;
use crate::modules::files::directory::DirectoryTask;
use crate::modules::files::file::FileTask;
use crate::modules::files::git::GitTask;
use crate::modules::files::stat::StatTask;
use crate::modules::files::template::TemplateTask;

// packages
use crate::modules::packages::apt::AptTask;
use crate::modules::packages::homebrew::HomebrewTask;
use crate::modules::packages::pacman::PacmanTask;
use crate::modules::packages::yum_dnf::YumDnfTask;
use crate::modules::packages::zypper::ZypperTask;

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
    Debug(DebugTask),
    Directory(DirectoryTask),
    Dnf(YumDnfTask),
    Echo(EchoTask),
    Facts(FactsTask),
    Fail(FailTask),
    File(FileTask),
    Git(GitTask),
    Homebrew(HomebrewTask),
    Pacman(PacmanTask),
    Sd_Service(SystemdServiceTask),
    Set(SetTask),
    Shell(ShellTask),
    Stat(StatTask),
    Template(TemplateTask),
    User(UserTask),
    Yum(YumDnfTask),
    Zypper(ZypperTask),
}

impl Task {

    pub fn get_module(&self) -> String {
        return match self {
            Task::Apt(x)        => x.get_module(),
            Task::Assert(x)     => x.get_module(),
            Task::Copy(x)       => x.get_module(),
            Task::Debug(x)      => x.get_module(),
            Task::Directory(x)  => x.get_module(),
            Task::Dnf(x)        => x.get_module(),
            Task::Echo(x)       => x.get_module(),
            Task::Facts(x)      => x.get_module(), 
            Task::Fail(x)       => x.get_module(), 
            Task::File(x)       => x.get_module(),
            Task::Git(x)        => x.get_module(), 
            Task::Homebrew(x)   => x.get_module(),
            Task::Pacman(x)     => x.get_module(),
            Task::Sd_Service(x) => x.get_module(),
            Task::Set(x)        => x.get_module(), 
            Task::Shell(x)      => x.get_module(), 
            Task::Stat(x)       => x.get_module(), 
            Task::Template(x)   => x.get_module(), 
            Task::User(x)       => x.get_module(),
            Task::Yum(x)        => x.get_module(),
            Task::Zypper(x)     => x.get_module(),
        };
    }

    pub fn get_name(&self) -> Option<String> {
        return match self {
            Task::Apt(x)        => x.get_name(),
            Task::Assert(x)     => x.get_name(),
            Task::Copy(x)       => x.get_name(),
            Task::Debug(x)      => x.get_name(), 
            Task::Directory(x)  => x.get_name(),
            Task::Dnf(x)        => x.get_name(),
            Task::Echo(x)       => x.get_name(),
            Task::Facts(x)      => x.get_name(),
            Task::Fail(x)       => x.get_name(), 
            Task::File(x)       => x.get_name(), 
            Task::Git(x)        => x.get_name(),
            Task::Homebrew(x)   => x.get_name(),
            Task::Pacman(x)     => x.get_name(),
            Task::Sd_Service(x) => x.get_name(),
            Task::Set(x)        => x.get_name(),
            Task::Shell(x)      => x.get_name(), 
            Task::Stat(x)       => x.get_name(),
            Task::Template(x)   => x.get_name(), 
            Task::User(x)       => x.get_name(),
            Task::Yum(x)        => x.get_name(),
            Task::Zypper(x)     => x.get_name(),
        };
    }

    pub fn get_with(&self) -> Option<PreLogicInput> {
        return match self {
            Task::Apt(x)        => x.get_with(),
            Task::Assert(x)     => x.get_with(),
            Task::Copy(x)       => x.get_with(),
            Task::Debug(x)      => x.get_with(), 
            Task::Directory(x)  => x.get_with(),
            Task::Dnf(x)        => x.get_with(),
            Task::Echo(x)       => x.get_with(),
            Task::Facts(x)      => x.get_with(),
            Task::Fail(x)       => x.get_with(), 
            Task::File(x)       => x.get_with(),
            Task::Git(x)        => x.get_with(), 
            Task::Homebrew(x)   => x.get_with(),
            Task::Pacman(x)     => x.get_with(),
            Task::Sd_Service(x) => x.get_with(),
            Task::Set(x)        => x.get_with(),
            Task::Shell(x)      => x.get_with(), 
            Task::Stat(x)       => x.get_with(), 
            Task::Template(x)   => x.get_with(),
            Task::User(x)       => x.get_with(),
            Task::Yum(x)        => x.get_with(), 
            Task::Zypper(x)     => x.get_with(),
        };
    }

    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        // ADD NEW MODULES HERE, KEEP ALPHABETIZED BY NAME
        return match self {
            Task::Apt(x)        => x.evaluate(handle, request, tm),
            Task::Assert(x)     => x.evaluate(handle, request, tm),
            Task::Copy(x)       => x.evaluate(handle, request, tm),
            Task::Debug(x)      => x.evaluate(handle, request, tm), 
            Task::Directory(x)  => x.evaluate(handle, request, tm), 
            Task::Dnf(x)        => x.evaluate(handle, request, tm),
            Task::Echo(x)       => x.evaluate(handle, request, tm),
            Task::Facts(x)      => x.evaluate(handle, request, tm),
            Task::Fail(x)       => x.evaluate(handle, request, tm),  
            Task::File(x)       => x.evaluate(handle, request, tm), 
            Task::Git(x)        => x.evaluate(handle, request, tm),
            Task::Homebrew(x)   => x.evaluate(handle, request, tm),
            Task::Pacman(x)     => x.evaluate(handle, request, tm),
            Task::Sd_Service(x) => x.evaluate(handle, request, tm),
            Task::Set(x)        => x.evaluate(handle, request, tm),
            Task::Shell(x)      => x.evaluate(handle, request, tm), 
            Task::Stat(x)       => x.evaluate(handle, request, tm),
            Task::Template(x)   => x.evaluate(handle, request, tm), 
            Task::User(x)       => x.evaluate(handle, request, tm),
            Task::Yum(x)        => x.evaluate(handle, request, tm), 
            Task::Zypper(x)     => x.evaluate(handle, request, tm), 
        };
    }

    // ==== END MODULE REGISTRY CONFIG ====

    pub fn get_display_name(&self) -> String {
        return match self.get_name() { Some(x) => x, _ => self.get_module()  }
    }

}




