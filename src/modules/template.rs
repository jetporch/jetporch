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
use std::path::{PathBuf};
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::collections::HashSet;

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
    pub src: PathBuf,
    pub dest: String,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for TemplateTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        let src = handle.template_string(&request, &String::from("src"), &self.src)?;
        return Ok(
            EvaluatedTask {
                action: Arc::new(TemplateAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    src:        handle.find_template_path(request, &String::from("src"), &src)?,
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
                let remote_mode = handle.remote_stat(request, &self.dest)?;
                if remote_mode.is_none() {
                    println!("XDEBUG: no stat");
                    return Ok(handle.needs_creation(&request));
                }
                else {
                    println!("DEBUG: got a stat {}", remote_mode.unwrap());
                }

                let mut changes : Option<Arc<HashSet<Field>>> = Some(Arc::new(HashSet::new()));
                /*
                let local_checksum = self.local_checksum(&self.src)?;
                handle.query_common_file_attributes(request, &self.dest, &remote_mode, &local_checksum, &mut changes)?;                   
                
                if changes.len() > 0 {
                    return self.handle.needs_modification(changes);
                }
                */
                return Ok(handle.is_matched(request));
            },

            TaskRequestType::Create => {
                /*
                handle.template_remote_file(request, &self.src, &self.dest)?
                handle.process_all_common_file_attributes(&request)?;
                */
                println!("ON CREATE!");
                let rc = handle.is_created(&request);
                println!("RETURNING: {:?}", rc);
                return Ok(rc);
            }

            TaskRequestType::Modify => {
                /*
                if changes.contains(String::from(Field.Checksum)) {
                    handle.template_remote_file(request, &self.src, &self.dest)?
                }
                handle.process_common_file_attributes(&request, &request.changes)?;
                */
                return Err(handle.not_supported(&request));
            }
    
            _ => { return Err(handle.not_supported(&request)); }
    
        }
    }

}
