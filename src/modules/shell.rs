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
use crate::connection::command::cmd_info;
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;

const MODULE: &'static str = "Shell";

#[derive(Deserialize,Debug)]
#[serde(tag="shell",deny_unknown_fields)]
pub struct ShellTask {
    pub name: Option<String>,
    pub cmd: String,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>,
    #[serde(rename = "unsafe")]
    pub unsafe_: Option<String>,
}
struct ShellAction {
    pub name: String,
    pub cmd: String,
    pub unsafe_: bool,
}


impl IsTask for ShellTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(ShellAction {
                    name: self.name.clone().unwrap_or(String::from(MODULE)),
                    unsafe_:  {
                        if self.cmd.find("{{").is_none() {
                            // allow all the fancy shell characters unless variables are used, in which case
                            // do a bit of extra filtering unless users turn it off
                            true
                        } else {
                            handle.template.boolean_option(&request, &String::from("unsafe"), &self.unsafe_)?
                        }
                    },
                    cmd:  handle.template.string_unsafe(&request, &String::from("cmd"), &self.cmd)?,
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, &self.and)?),
            }
        );
    }

}

impl IsAction for ShellAction {
    
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_execution(&request));
            },

            TaskRequestType::Execute => {
                let mut task_result : Arc<TaskResponse>;
                if self.unsafe_ {
                    task_result = handle.remote.run_unsafe(&request, &self.cmd.clone(), CheckRc::Unchecked)?;
                } else {
                    task_result = handle.remote.run(&request, &self.cmd.clone(), CheckRc::Unchecked)?;
                }
                let (rc, _out) = cmd_info(&task_result);
                return match rc {
                    0 => Ok(task_result), 
                    _ => Err(handle.response.command_failed(request, &Arc::clone(&task_result.command_result)))
                };
            },
    
            _ => { return Err(handle.response.not_supported(&request)); }
    
        }
    }

}
