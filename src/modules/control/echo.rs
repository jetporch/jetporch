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
//#[allow(unused_imports)]
use serde::Deserialize;
use std::sync::Arc;

const MODULE: &'static str = "Echo";

#[derive(Deserialize,Debug)]
#[serde(tag="echo",deny_unknown_fields)]
pub struct EchoTask {
    pub name: Option<String>,
    pub msg: String,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
#[allow(dead_code)]
struct EchoAction {
    pub name: String,
    pub msg: String,
}

impl IsTask for EchoTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(EchoAction {
                    name: self.name.clone().unwrap_or(String::from(MODULE)),
                    msg:  handle.template.string_unsafe(request, &String::from("msg"), &self.msg)?,
                }),
                with: Arc::new(PreLogicInput::template(handle, request, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, &self.and)?),
            }
        );
    }
}

impl IsAction for EchoAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                handle.debug(&request, &self.msg);
                return Ok(handle.response.is_passive(request));
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }

    }

}
