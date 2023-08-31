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
use std::path::{PathBuf};
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &'static str = "File";

#[derive(Deserialize,Debug)]
#[serde(tag="file",deny_unknown_fields)]
pub struct FileTask {
    pub name: Option<String>,
    pub path: String,
    pub delete: Option<bool>,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct FileAction {
    pub name: String,
    pub path: PathBuf,
    pub delete: bool,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for CopyTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(FileAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    delete:     handle.template.boolean_option(&request, &String::from("delete"), &self.delete)?,
                    path:       handle.template.path(&request, &String::from("path"), &self.path)?,
                    attributes: FileAttributesInput::template(&handle, &request, &self.attributes)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}

impl IsAction for FileAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                let mut changes : Vec<Field> = Vec::new();
                let remote_mode = handle.remote.query_common_file_attributes(request, &self.dest, &self.attributes, &mut changes)?;                   
                if remote_mode.is_none() {
                    if self.delete             { return Ok(handle.response.is_matched(request)); } 
                    else                       { return Ok(handle.response.needs_creation(request));  }
                } else {
                    let is_dir = handle.remote.get_is_directory(request, &self.dest)?;
                    if is_dir                  { return Err(handle.response.is_failed(request, &format!("{} is a directory", self.dest))); }
                    else if self.delete        { return Ok(handle.response.needs_removal(request)); }
                    else if changes.is_empty() { return Ok(handle.response.is_matched(request)); }
                    else                       { return Ok(handle.response.needs_modification(request, &changes)); }
                }
            },

            TaskRequestType::Create => {
                self.remote.touch_file(handle, request)?;               
                handle.remote.process_all_common_file_attributes(request, &self.dest, &self.attributes)?;
                return Ok(handle.is_created(request));
            },

            TaskRequestType::Modify => {
                handle.remote.process_common_file_attributes(request, &self.dest, &self.attributes, &request.changes)?;
                return Ok(handle.is_modified(request, request.changes.clone()));
            },

            TaskRequestType::Remove => {
                self.remote.delete_file(handle, request)?;               
                return Ok(handle.is_removed(request))
            }

            // no passive or execute leg
            _ => { return Err(handle.not_supported(request)); }

        
        }
    }
}
