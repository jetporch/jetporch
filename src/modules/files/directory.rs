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
use crate::handle::handle::TaskHandle;
use crate::tasks::fields::Field;
use crate::tasks::files::Recurse;
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &'static str = "directory";

#[derive(Deserialize,Debug)]
#[serde(tag="directory",deny_unknown_fields)]
pub struct DirectoryTask {
    pub name: Option<String>,
    pub path: String,
    pub remove: Option<String>,
    pub recurse: Option<String>,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct DirectoryAction {
    pub name: String,
    pub path: String,
    pub remove: bool,
    pub recurse: Recurse,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for DirectoryTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        let recurse = match handle.template.boolean_option_default_false(&request, &String::from("recurse"), &self.recurse)? {
            true => Recurse::Yes,
            false => Recurse::No
        };
        return Ok(
            EvaluatedTask {
                action: Arc::new(DirectoryAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    remove:     handle.template.boolean_option_default_false(&request, &String::from("remove"), &self.remove)?,
                    recurse:    recurse, 
                    path:       handle.template.path(&request, &String::from("path"), &self.path)?,
                    attributes: FileAttributesInput::template(&handle, &request, &self.attributes)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}

impl IsAction for DirectoryAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                let mut changes : Vec<Field> = Vec::new();
                let remote_mode = handle.remote.query_common_file_attributes(request, &self.path, &self.attributes, &mut changes, self.recurse)?;                 
                if remote_mode.is_none() {
                    if self.remove             { return Ok(handle.response.is_matched(request)); } 
                    else                       { return Ok(handle.response.needs_creation(request));  }
                } else {
                    let is_file = handle.remote.get_is_file(request, &self.path)?;
                    if is_file                 { return Err(handle.response.is_failed(request, &format!("{} is not a directory", self.path))); }
                    else if self.remove        { return Ok(handle.response.needs_removal(request)); }
                    else if changes.is_empty() { return Ok(handle.response.is_matched(request)); }
                    else                       { return Ok(handle.response.needs_modification(request, &changes)); }
                }
            },

            TaskRequestType::Create => {
                handle.remote.create_directory(request, &self.path)?;               
                handle.remote.process_all_common_file_attributes(request, &self.path, &self.attributes, self.recurse)?;
                return Ok(handle.response.is_created(request));
            },

            TaskRequestType::Modify => {
                handle.remote.process_common_file_attributes(request, &self.path, &self.attributes, &request.changes, self.recurse)?;
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            },

            TaskRequestType::Remove => {
                handle.remote.delete_directory(request, &self.path, self.recurse)?;               
                return Ok(handle.response.is_removed(request))
            }

            // no passive or execute leg
            _ => { return Err(handle.response.not_supported(request)); }

        
        }
    }
}