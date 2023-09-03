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
use crate::tasks::checksum::sha512;
use crate::tasks::fields::Field;
use std::path::{PathBuf};
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;
use crate::tasks::files::Recurse;

const MODULE: &'static str = "template";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct TemplateTask {
    pub name: Option<String>,
    pub src: String,
    pub dest: String,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct TemplateAction {
    pub src: PathBuf,
    pub dest: String,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for TemplateTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        let src = handle.template.string(&request, &String::from("src"), &self.src)?;
        return Ok(
            EvaluatedTask {
                action: Arc::new(TemplateAction {
                    src:        handle.template.find_template_path(request, &String::from("src"), &src)?,
                    dest:       handle.template.path(&request, &String::from("dest"), &self.dest)?,
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

                let mut changes : Vec<Field> = Vec::new();
                let remote_mode = handle.remote.query_common_file_attributes(request, &self.dest, &self.attributes, &mut changes, Recurse::No)?;                   
                if remote_mode.is_none() {
                    return Ok(handle.response.needs_creation(request));
                }
                let data = self.do_template(handle, request, false)?;
                let local_512 = sha512(&data);
                let remote_512 = handle.remote.get_sha512(request, &self.dest)?;
                if ! remote_512.eq(&local_512) { 
                    changes.push(Field::Content); 
                }
                if ! changes.is_empty() {
                    return Ok(handle.response.needs_modification(request, &changes));
                }
                return Ok(handle.response.is_matched(request));
            },

            TaskRequestType::Create => {
                self.do_template(handle, request, true)?;               
                handle.remote.process_all_common_file_attributes(request, &self.dest, &self.attributes, Recurse::No)?;
                return Ok(handle.response.is_created(request));
            }

            TaskRequestType::Modify => {
                if request.changes.contains(&Field::Content) {
                    self.do_template(handle, request, true)?;
                }
                handle.remote.process_common_file_attributes(request, &self.dest, &self.attributes, &request.changes, Recurse::No)?;
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            }
    
            _ => { return Err(handle.response.not_supported(request)); }
    
        }
    }

}

impl TemplateAction {

    pub fn do_template(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, write: bool) -> Result<String, Arc<TaskResponse>> {
        let template_contents = handle.local.read_file(&request, &self.src)?;
        let data = handle.template.string_for_template_module_use_only(&request, &String::from("src"), &template_contents)?;
        if write {
            handle.remote.write_data(&request, &data, &self.dest)?;
        }
        return Ok(data);
    }

}
