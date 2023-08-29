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
use crate::tasks::checksum::sha512;
//use std::path::Path;

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
                    dest:       handle.template_path(&request, &String::from("dest"), &self.dest)?,
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

                let mut changes : HashSet<Field> = HashSet::new();
                let remote_mode = handle.query_common_file_attributes(request, &self.dest, &self.attributes, &mut changes)?;                   
                if remote_mode.is_none() {
                    return Ok(handle.needs_creation(&request));
                }
                
                // check for modifications needed
                //let src_path = self.src.as_path();
                let data = self.do_template(&handle, &request, false)?;
                // DON'T DELETE THIS YET - the copy module will want for it!
                //let local_512 = handle.get_local_sha512(request, &src_path, true)?;
                let local_512 = sha512(&data);
                let remote_512 = handle.get_remote_sha512(request, &self.dest)?;
                if ! remote_512.eq(&local_512) { 
                    changes.insert(Field::Content); 
                }

                if ! changes.is_empty() {
                    return Ok(handle.needs_modification(&request, &changes));
                }
                
                return Ok(handle.is_matched(&request));
            },

            TaskRequestType::Create => {
                self.do_template(handle, request, true)?;               
                // handle.process_all_common_file_attributes(&request, &self.attributes)?;
                let rc = handle.is_created(&request);
                return Ok(rc);
            }

            TaskRequestType::Modify => {

                let changes = request.changes.clone();

                if changes.contains(&Field::Content) {
                    println!("MAKING CHANGES!");
                    self.do_template(&handle, &request, true)?;
                } else {
                    println!("CONTENT LOOKS GOOD!");
                }
                /*
                if changes.contains(String::from(Field.Checksum)) {
                    handle.template_remote_file(request, &self.src, &self.dest)?
                }
                handle.process_common_file_attributes(&request, &request.changes)?;
                */
                return Ok(handle.is_modified(&request, changes));
            }
    
            _ => { return Err(handle.not_supported(&request)); }
    
        }
    }

}

impl TemplateAction {

    pub fn do_template(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, write: bool) -> Result<String, Arc<TaskResponse>> {
        let remote_put_mode = handle.get_desired_numeric_mode(&request, &self.attributes)?;
        let template_contents = handle.read_local_file(&request, &self.src)?;
        let data = handle.template_string(&request, &String::from("src"), &template_contents)?;
        if write {
            handle.write_remote_data(&request, &data, &self.dest, remote_put_mode)?;
        }
        return Ok(data);
    }

}
