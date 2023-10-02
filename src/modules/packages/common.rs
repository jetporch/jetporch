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
use crate::handle::handle::{TaskHandle};
use crate::tasks::fields::Field;
use std::sync::Arc;

#[derive(Clone,PartialEq,Debug)]
pub struct PackageDetails {
    pub name: String,
    pub version: String,
}

pub trait PackageManagementModule {

    fn is_update(&self) -> bool;
    fn is_remove(&self) -> bool;
    fn get_version(&self) -> Option<String>;

    fn initial_setup(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<(),Arc<TaskResponse>>;
            
    fn get_local_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>>;
                
    fn get_remote_version(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Option<PackageDetails>,Arc<TaskResponse>>;

    fn install_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>;
    
    fn update_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>;
    
    fn remove_package(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>;
    
    fn common_package_query(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
        
        let mut changes : Vec<Field> = Vec::new();

        self.initial_setup(handle, request)?;
        
        let package_details = self.get_local_version(handle, request)?; 

        if package_details.is_some() {
            // package is installed
            if self.is_remove() {
                return Ok(handle.response.needs_removal(request));
            }
            let pkg = package_details.unwrap();
            let version = self.get_version();

            if self.is_update() {
                let remote_details = self.get_remote_version(handle, request)?;
                if remote_details.is_none() || !pkg.version.eq(&remote_details.unwrap().version) {
                    changes.push(Field::Version);
                }
            } else if version.is_some() {
                let specified_version = version.as_ref().unwrap();
                if ! pkg.version.eq(specified_version) { changes.push(Field::Version); }
            }
        
            if changes.len() > 0 {
                return Ok(handle.response.needs_modification(request, &changes));
            } else {
                return Ok(handle.response.is_matched(request));
            }
        } else {
            // package is not installed
            return match self.is_remove() {
                true => Ok(handle.response.is_matched(request)),
                false => Ok(handle.response.needs_creation(request))
            }
        }    
    }

    fn common_dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return self.common_package_query(handle, request);
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