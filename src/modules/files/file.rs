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
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;
use crate::tasks::files::Recurse;

const MODULE: &'static str = "file";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FileTask {
    pub name: Option<String>,
    pub path: String,
    pub remove: Option<String>,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct FileAction {
    pub path: String,
    pub remove: bool,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for FileTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(FileAction {
                    remove:     handle.template.boolean_option_default_false(&request, tm, &String::from("remove"), &self.remove)?,
                    path:       handle.template.path(&request, tm, &String::from("path"), &self.path)?,
                    attributes: FileAttributesInput::template(&handle, &request, tm, &self.attributes)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for FileAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                let mut changes : Vec<Field> = Vec::new();
                let remote_mode = handle.remote.query_common_file_attributes(request, &self.path, &self.attributes, &mut changes, Recurse::No)?;                   
                if remote_mode.is_none() {
                    if self.remove             { return Ok(handle.response.is_matched(request)); } 
                    else                       { return Ok(handle.response.needs_creation(request));  }
                } else {
                    let is_dir = handle.remote.get_is_directory(request, &self.path)?;
                    if is_dir                  { return Err(handle.response.is_failed(request, &format!("{} is a directory", self.path))); }
                    else if self.remove        { return Ok(handle.response.needs_removal(request)); }
                    else if changes.is_empty() { return Ok(handle.response.is_matched(request)); }
                    else                       { return Ok(handle.response.needs_modification(request, &changes)); }
                }
            },

            TaskRequestType::Create => {
                handle.remote.touch_file(request, &self.path)?;               
                handle.remote.process_all_common_file_attributes(request, &self.path, &self.attributes, Recurse::No)?;
                return Ok(handle.response.is_created(request));
            },

            TaskRequestType::Modify => {
                handle.remote.process_common_file_attributes(request, &self.path, &self.attributes, &request.changes, Recurse::No)?;
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            },

            TaskRequestType::Remove => {
                handle.remote.delete_file(request, &self.path)?;               
                return Ok(handle.response.is_removed(request))
            }

            // no passive or execute leg
            _ => { return Err(handle.response.not_supported(request)); }

        
        }
    }
}
