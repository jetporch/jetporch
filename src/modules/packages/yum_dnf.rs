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
use crate::inventory::hosts::PackagePreference;
use serde::{Deserialize};
use std::sync::Arc;

const MODULE: &str = "yum_dnf";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct YumDnfTask {
    pub name: Option<String>,
    pub package: String,
    pub version: Option<String>,
    pub update: Option<String>,
    pub remove: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct YumDnfAction {
    pub package: String,
    pub version: Option<String>,
    pub update: bool,
    pub remove: bool,
}

impl IsTask for YumDnfTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(YumDnfAction {
                    package:    handle.template.string_no_spaces(request, tm, &String::from("package"), &self.package)?,
                    version:    handle.template.string_option_no_spaces(&request, tm, &String::from("version"), &self.version)?,
                    update:     handle.template.boolean_option_default_false(&request, tm, &String::from("update"), &self.update)?,
                    remove:     handle.template.boolean_option_default_false(&request, tm, &String::from("remove"), &self.remove)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for YumDnfAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
        return self.common_dispatch(handle,request);
    }

}

impl PackageManagementModule for YumDnfAction {

    fn is_update(&self) -> bool {
        return self.update;
    }

    fn is_remove(&self) -> bool {
        return self.remove; 
    }

    fn get_version(&self) -> Option<String> {
        return self.version.clone();
    }

    fn initial_setup(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<(),Arc<TaskResponse>> {
        self.set_package_preference(handle,request)?;
        return Ok(());
    }

    fn get_local_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let which = self.get_package_manager(handle);
        let cmd = match self.version.is_none() {
            true => format!("{} info {}", which, self.package),
            false => format!("{} info {}-{}", which, self.package, self.version.as_ref().unwrap())
        };
        let result = handle.remote.run(request, &cmd, CheckRc::Unchecked)?;
        let (_rc,out) = cmd_info(&result);
        let details = self.parse_local_package_details(&out.clone())?;
        return Ok(details);
    }

    fn get_remote_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let cmd = format!("repoquery {} --queryformat {}", self.package, "'%{version}'");
        let result = handle.remote.run_unsafe(request, &cmd, CheckRc::Unchecked)?;
        let (_rc,out) = cmd_info(&result);
        let details = self.parse_remote_package_details(&out.clone())?;
        return Ok(details);
    }
    
    fn install_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let which = self.get_package_manager(handle);
        let cmd = match self.version.is_none() {
            true => format!("{} install '{}' -y", which, self.package),
            false => format!("{} install '{}-{}' -y", which, self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let which = self.get_package_manager(handle);
        let cmd = format!("{} update '{}' -y", which, self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let which = self.get_package_manager(handle);
        let cmd = format!("{} remove '{}' -y", which, self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

}

impl YumDnfAction {

    pub fn set_package_preference(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<(),Arc<TaskResponse>> {
        if handle.host.read().unwrap().package_preference.is_some() {
            return Ok(());
        }
        match handle.remote.get_mode(request, &String::from("/usr/bin/dnf"))? {
            Some(_) => {
                handle.host.write().unwrap().package_preference = Some(PackagePreference::Dnf);
            }
            None => match handle.remote.get_mode(request, &String::from("/usr/bin/yum"))? {
                Some(_) => {
                    handle.host.write().unwrap().package_preference = Some(PackagePreference::Yum);
                }
                None => { return Err(handle.response.is_failed(request, &String::from("neither dnf nor yum detected"))); }
            }
        }
        Ok(())
    }

    pub fn get_package_preference(&self, handle: &Arc<TaskHandle>) -> Option<PackagePreference> {
        handle.host.read().unwrap().package_preference
    }

    pub fn get_package_manager(&self, handle: &Arc<TaskHandle>) -> String {
        match self.get_package_preference(handle) {
            Some(PackagePreference::Yum) => String::from("yum"),
            Some(PackagePreference::Dnf) => String::from("dnf"),
            _ => { panic!("internal error, package preference not set correctly"); }
        }
    }

    fn parse_local_package_details(&self, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let mut name: Option<String> = None;
        let mut version: Option<String> = None;
        for line in out.lines() {
            if line.starts_with("Available") {
                return Ok(None);
            }
            let mut tokens = line.split(":");
            let key = tokens.nth(0);
            let value = tokens.nth(0);
            if key.is_some() && value.is_some() {
                let key2 = key.unwrap().trim();
                let value2 = value.unwrap().trim();
                if key2.eq("Name")    { name = Some(value2.to_string());           }
                if key2.eq("Version") { version = Some(value2.to_string()); break; }
            }
        }
        if name.is_some() && version.is_some() {
            return Ok(Some(PackageDetails { name: name.unwrap().clone(), version: version.unwrap().clone() }));
        } else {
            return Ok(None);
        }
    }
    
    fn parse_remote_package_details(&self, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        // FYI: this command doesn't have useful return codes
        for line in out.lines() {
            if ! line.contains("metadata expiration") {
                return Ok(Some(PackageDetails {
                    name: self.package.clone(),
                    version: line.trim().to_string(),
                }));
            }
        }
        return Ok(None);
    }

}
