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

const MODULE: &str = "apt";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct AptTask {
    pub name: Option<String>,
    pub package: String,
    pub version: Option<String>,
    pub update: Option<String>,
    pub remove: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct AptAction {
    pub package: String,
    pub version: Option<String>,
    pub update: bool,
    pub remove: bool,
}

impl IsTask for AptTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(AptAction {
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

impl IsAction for AptAction {
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
        return self.common_dispatch(handle,request);
    }
}

impl PackageManagementModule for AptAction {

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
        /* need to implement so update returns the correct modification status */
        let cmd = format!("apt-cache show {} | grep Version | cut -f2 --delimiter=':'", self.package.clone());
        let result = handle.remote.run_unsafe(request, &cmd, CheckRc::Unchecked);
        match result {
            Ok(r) => {
                let (rc,out) = cmd_info(&r);
                // (differentiated return code is unavailable for this module only, don't repeat this pattern)
                if out.contains("No packages found") {
                    return Ok(None);
                }
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
        let cmd = format!("dpkg-query -W '{}'", self.package);
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
            true => format!("DEBIAN_FRONTEND=noninteractive apt-get install '{}' -qq", self.package),
            false => format!("DEBIAN_FRONTEND=noninteractive apt-get install '{}={}' -qq", self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = match self.version.is_none() {
            true => format!("DEBIAN_FRONTEND=noninteractive apt-get install '{}' --only-upgrade -qq", self.package),
            false => format!("DEBIAN_FRONTEND=noninteractive apt-get install '{}={}' --only-upgrade -qq", self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("DEBIAN_FRONTEND=noninteractive apt-get remove '{}' -qq", self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

}

impl AptAction {

    pub fn parse_local_package_details(&self, _handle: &Arc<TaskHandle>, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let mut tokens = out.split("\t");
        let version = tokens.nth(1);
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
