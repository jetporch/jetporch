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

const MODULE: &'static str = "Dnf";

#[derive(Deserialize,Debug)]
#[serde(tag="dnf",deny_unknown_fields)]
pub struct DnfTask {
    pub name: Option<String>,
    pub package: String,
    pub version: String,
    pub update: Option<bool>,
    pub remove: Option<bool>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct DnfAction {
    pub name: String,
    pub package: String,
    pub version: Option<String>,
    pub update: Option<String>,
    pub remove: bool,
}

#[derive(Copy,Clone,PartialEq,Debug)]
struct PackageDetails {
    name: String,
    version: Version,
}

impl IsTask for DnfTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(TemplateAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    package:    handle.template.string_no_spaces(request, &String::from("package"), &package)?),
                    version:    handle.template.string_option_no_spaces(&request, &String::from("version"), &self.version)?),
                    update:     handle.template.boolean_option(&request, &String::from("update"), &self.update)?
                    remove:     handle.template.boolean_option(&request, &String::from("remove"), &self.remove)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}


impl IsAction for DnfAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {

                let mut changes : Vec<Field> = Vec::new();
                let package_details = self.get_package_details(handle, request, &self.package)?;   
                if package_details.is_some() {
                    // package is installed
                    if self.remove {
                        return Ok(handle.response.needs_deletion(request));
                    }
                    let pkg = package_details.unwrap();
                    if self.update || (self.version.is_some() && ! pkg.version.eq(self.version.unwrap())) { 
                        changes.push(Field.Version);
                    }
                    if changes.len() > 0 {
                        return Ok(handle.response.needs_modification(request, &changes));
                    } else {
                        return Ok(handle.response.is_matched(request));
                    }
                }
                else {
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
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            }
    
            _ => { return Err(handle.response.not_supported(request)); }
    
        }
    }

}

impl DnfAction {

    pub fn get_package_details(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Option<PackageDetails,Arc<TaskResponse>> {
        let cmd = match self.version.is_none() {
            true => format!("dnf show {}", self.package);
            false => format!("dnf show {}-{}", self.package, self.version.unwrap());
        }
        let result = handle.remote.run(request, cmd, CheckRc::Yes)?;
        (rc,out) = cmd_info(result);
        return Ok(parse_package_details(&out.clone());
    }

    pub fn parse_package_details(out: &String) -> Result<Option<PackageDetails>,Arc<TaskResponse>> {
        let mut name: Option<String> = None;
        let mut version: Option<String> = None;
        for line in out.lines() {
            if line.starts_with("Available") {
                return None;
            }
            let tokens = line.split(":");
            let key = tokens.nth(0);
            let value = tokens.nth(0);
            if key.is_some() and value.is_some() {
                let key2 = key.trim();
                let value2 = value2.trim();
                if key2.eq("Name")    { *name = Some(value2.clone());           }
                if key2.eq("Version") { *version = Some(value2.clone()); break; }
            }
        }
        if name.is_some() and version.is_some() {
            return Some(PackageDetails { name: name.clone(), version: version.clone() });
        } else {
            return None;
        }
    }

    pub fn install_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Option<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = match self.version.is_none() {
            true => format!("dnf install {}", self.package),
            false => format!("dnf install {}-{}", self.package, self.version.unwrap())
        };
        return self.run(request, cmd, CheckRc::Yes);
    }

    pub fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Option<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = format!("dnf update {}", self.package);
        return self.run(request, cmd, CheckRc::Yes);
    }

    pub fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Option<Arc<TaskResponse>,Arc<TaskResponse>>{
        let cmd = format!("dnf remove {}", self.package);
        return self.run(request, cmd, CheckRc::Yes);
    }

}
