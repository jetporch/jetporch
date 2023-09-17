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
use crate::handle::handle::{TaskHandle,CheckRc};
use crate::tasks::fields::Field;
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &'static str = "dnf";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct DnfTask {
    pub name: Option<String>,
    pub package: String,
    pub version: Option<String>,
    pub update: Option<String>,
    pub remove: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct DnfAction {
    pub package: String,
    pub version: Option<String>,
    pub update: bool,
    pub remove: bool,
}

#[derive(Clone,PartialEq,Debug)]
struct PackageDetails {
    name: String,
    version: String,
}

impl IsTask for DnfTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(DnfAction {
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


impl IsAction for DnfAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {

                // FIXME: ALL of this query logic is shared between dnf and apt, but it is likely other package managers
                // will diverge.  Still, consider a common function.

                let mut changes : Vec<Field> = Vec::new();
                let package_details = self.get_package_details(handle, request)?; 

                if package_details.is_some() {
                    // package is installed
                    if self.remove {
                        return Ok(handle.response.needs_removal(request));
                    }
                    let pkg = package_details.unwrap();

                    if self.update {
                        changes.push(Field::Version);
                    } else if self.version.is_some() {
                        let specified_version = self.version.as_ref().unwrap();
                        if ! pkg.version.eq(specified_version) { changes.push(Field::Version); }
                    }

                    if changes.len() > 0 {
                        return Ok(handle.response.needs_modification(request, &changes));
                    } else {
                        return Ok(handle.response.is_matched(request));
                    }
                } else {
                    // package is not installed
                    return match self.remove {
                        true => Ok(handle.response.is_matched(request)),
                        false => Ok(handle.response.needs_creation(request))
                    }
                }
            },

            TaskRequestType::Create => {
                self.install_package(handle, request)?;               
                return Ok(handle.response.is_created(request));
            }

            TaskRequestType::Modify => {
                if request.changes.contains(&Field::Version) {
                    self.update_package(handle, request)?;
                }
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            }

            TaskRequestType::Remove => {
                self.remove_package(handle, request)?;
                return Ok(handle.response.is_removed(request));
            }
    
            _ => { return Err(handle.response.not_supported(request)); }
    
        }
    }

}

impl DnfAction {

    pub fn get_package_details(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let cmd = match self.version.is_none() {
            true => format!("dnf info {}", self.package),
            false => format!("dnf info {}-{}", self.package, self.version.as_ref().unwrap())
        };
        let result = handle.remote.run(request, &cmd, CheckRc::Unchecked)?;
        let (_rc,out) = cmd_info(&result);
        let details = self.parse_package_details(&out.clone())?;
        return Ok(details);
    }

    pub fn parse_package_details(&self, out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
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

    pub fn install_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = match self.version.is_none() {
            true => format!("dnf install '{}' -y", self.package),
            false => format!("dnf install '{}-{}' -y", self.package, self.version.as_ref().unwrap())
        };
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    pub fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = format!("dnf update '{}' -y", self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    pub fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = format!("dnf remove '{}' -y", self.package);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

}
