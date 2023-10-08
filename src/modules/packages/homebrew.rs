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

use crate::tasks::*;
use crate::modules::packages::common::{PackageManagementModule,PackageDetails};
use crate::handle::handle::{TaskHandle,CheckRc};
use serde::{Deserialize};
use std::sync::Arc;

const MODULE: &str = "brew";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct HomebrewTask {
    pub name: Option<String>,
    pub package: String,
    pub version: Option<String>,
    pub update: Option<String>,
    pub remove: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct HomebrewAction {
    pub package: String,
    pub version: Option<String>,
    pub update: bool,
    pub remove: bool,
}

impl IsTask for HomebrewTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(HomebrewAction {
                    package:    handle.template.string_no_spaces(request, tm, &String::from("package"), &self.package)?,
                    version:    handle.template.string_option_no_spaces(&request, tm, &String::from("version"), &self.version)?,
                    update:     handle.template.boolean_option_default_false(&request, tm, &String::from("update"), &self.update)?,
                    remove:     handle.template.boolean_option_default_false(&request, tm, &String::from("remove"), &self.remove)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?)
            }
        );
    }

}

impl IsAction for HomebrewAction {
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
        return self.common_dispatch(handle,request);
    }
}

impl PackageManagementModule for HomebrewAction {

    fn initial_setup(&self, _handle: &Arc<TaskHandle>, _request: &Arc<TaskRequest>) -> Result<(),Arc<TaskResponse>> {
        // nothing to do here, see how this was used in yum_dnf.rs
        return Ok(());
    }

    fn is_update(&self) -> bool {
        return self.update;
    }

    fn is_remove(&self) -> bool {
        return self.remove;
    }

    fn get_version(&self) -> Option<String> {
        return self.version.clone();
    }

    fn get_remote_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let cmd = format!("brew info {} | head -n 1 | cut -f 4 -d ' '", self.package.clone());
        let result = handle.remote.run_unsafe(request, &cmd, CheckRc::Unchecked);
        match result {
            Ok(r) => {
                let (rc,out) = cmd_info(&r);
                if rc == 0 {
                    let details = self.parse_remote_package_details(handle, &out.clone());
                    return details;
                } else {
                    return Ok(None);
                }
            },
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn get_local_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let cmd = format!("brew info {}", self.package);
        let result = handle.remote.run(request, &cmd, CheckRc::Unchecked);
        match result {
            Ok(r) => {
                let (rc,out) = cmd_info(&r);
                if rc == 0 {
                    let details = self.parse_local_package_details(handle, &out.clone())?;
                    return Ok(details);
                } else {
                    return Ok(None);
                }
            },
            Err(e) => return Err(e)
        }
    }

    fn install_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = match self.version.is_none() {
            true => format!("brew install '{}'", self.package),
            false => format!("brew install '{}@{}'", self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = match self.version.is_none() {
            true => format!("brew upgrade '{}'", self.package),
            false => format!("brew upgrade '{}@{}'", self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("brew uninstall '{}'", self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

}

impl HomebrewAction {

    pub fn parse_local_package_details(&self, _handle: &Arc<TaskHandle>, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let mut version: Option<String> = None;

        for line in out.lines() {
            if line.starts_with('/') {
                let path_version = line.split(" ").nth(0).unwrap().trim();
                version = Some(path_version.split("/").last().unwrap().trim().to_string());
            }
            // homebrew puts an asterisk next to the currently installed version
            if line.trim().ends_with('*') {
                break;
            }
            // once homebrew has printed the versions, it starts printing other stuff
            if line.starts_with("From:") {
                break;
            }
        }
        return match version {
            Some(v) => {
                Ok(Some(PackageDetails { name: self.package.clone(), version: v.trim().to_string() }))
            },
            None => {
                Ok(None)
            }
        };
    }

    pub fn parse_remote_package_details(&self, _handle: &Arc<TaskHandle>, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        return Ok(Some(PackageDetails { name: self.package.clone(), version: out.trim().to_string() }));
    }

}
