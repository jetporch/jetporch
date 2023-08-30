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
use crate::tasks::fields::Field;
use std::path::{PathBuf};
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &'static str = "Copy";

#[derive(Deserialize,Debug)]
#[serde(tag="copy",deny_unknown_fields)]
pub struct CopyTask {
    pub name: Option<String>,
    pub src: String,
    pub dest: String,
    pub attributes: Option<FileAttributesInput>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct CopyAction {
    pub name: String,
    pub src: PathBuf,
    pub dest: String,
    pub attributes: Option<FileAttributesEvaluated>,
}

impl IsTask for CopyTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        let src = handle.template_string(&request, &String::from("src"), &self.src)?;
        return Ok(
            EvaluatedTask {
                action: Arc::new(CopyAction {
                    name:       self.name.clone().unwrap_or(String::from(MODULE)),
                    src:        handle.find_file_path(request, &String::from("src"), &src)?,
                    dest:       handle.template_path(&request, &String::from("dest"), &self.dest)?,
                    attributes: FileAttributesInput::template(&handle, &request, &self.attributes)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}

impl IsAction for CopyAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {

                let mut changes : Vec<Field> = Vec::new();
                let remote_mode = handle.query_common_file_attributes(request, &self.dest, &self.attributes, &mut changes)?;                   
                if remote_mode.is_none() {
                    return Ok(handle.needs_creation(request));
                }
                // this query leg is (at least originally) the same as the template module query except these two lines
                // to calculate the checksum differently
                let src_path = self.src.as_path();
                let local_512 = handle.get_local_sha512(request, &src_path, true)?;
                let remote_512 = handle.get_remote_sha512(request, &self.dest)?;
                if ! remote_512.eq(&local_512) { 
                    changes.push(Field::Content); 
                }
                if ! changes.is_empty() {
                    return Ok(handle.needs_modification(request, &changes));
                }
                return Ok(handle.is_matched(request));
            },

            TaskRequestType::Create => {
                self.do_copy(handle, request)?;               
                handle.process_all_common_file_attributes(request, &self.dest, &self.attributes)?;
                return Ok(handle.is_created(request));
            }

            TaskRequestType::Modify => {
                if request.changes.contains(&Field::Content) {
                    self.do_copy(handle, request)?;
                }
                handle.process_common_file_attributes(request, &self.dest, &self.attributes, &request.changes)?;
                return Ok(handle.is_modified(request, request.changes.clone()));
            }
    
            _ => { return Err(handle.not_supported(request)); }
    
        }
    }

}

impl CopyAction {

    pub fn do_copy(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<(), Arc<TaskResponse>> {
        let remote_put_mode = handle.get_desired_numeric_mode(request, &self.attributes)?;
        handle.copy_file(request, &self.src, &self.dest, remote_put_mode)?;
        return Ok(());
    }

}
