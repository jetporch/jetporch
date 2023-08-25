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
//#[allow(unused_imports)]
use serde::{Deserialize};

const MODULE: &'static str = "Template";

#[derive(Deserialize,Debug)]
#[serde(tag="template",deny_unknown_fields)]
pub struct TemplateTask {
    pub name: Option<String>,
    pub src: String,
    pub dest: String,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct TemplateAction {
    pub name: String,
    pub src: String,
    pub dest: String,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for TemplateTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        let src_path_str = handle.template_string(&request, &String::from("src"), &self.src)?;
        return Ok(
            EvaluatedTask {
                action: Arc::new(TemplateAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    src:        src_path_str.clone(), // FIXME: working on it! handle.find_path(&request, SearchPaths::Templates, src_path_str)?,
                    dest:       handle.template_string(&request, &String::from("dest"), &self.dest)?,
                    attributes: FileAttributesInput::template(&handle, &request, &self.attributes)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}

impl IsAction for TemplateAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {


            TaskRequestType::Query => {
                // FIXME -- return is_matched or not
                // see if file exists, sha1sum, modes/etc etc, common tools in tasks/files.rs make sense
                // Ok(handle.needs_creation(&request));
                return Err(handle.not_supported(&request));
            },

            TaskRequestType::Create => {
                // FIXME
                return Ok(handle.is_created(&request));
            }

            TaskRequestType::Modify => {
                // FIXME -- requests.changes should be a hashset
                return Err(handle.not_supported(&request));

                //return Ok(handle.is_modified(&request, request.changes));

            }
            // NeedsModification


    
            _ => { return Err(handle.not_supported(&request)); }
    
        }
    }

}
