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

const MODULE: &'static str = "fail";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FailTask {
    pub name: Option<String>,
    pub msg: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

#[allow(dead_code)]
struct FailAction {
    pub name: String,
    pub msg: Option<String>,
}

impl IsTask for FailTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(FailAction {
                    name: self.name.clone().unwrap_or(String::from(MODULE)),
                    msg:  handle.template.string_option_unsafe(request, tm, &String::from("msg"), &self.msg)?,
                }),
                with: Arc::new(PreLogicInput::template(handle, request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, tm, &self.and)?),
            }
        );
    }
}

impl IsAction for FailAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                let msg = match self.msg.is_some() {
                    true => self.msg.as_ref().unwrap().clone(),
                    false => String::from("fail invoked")
                };
                return Err(handle.response.is_failed(request, &msg));
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }

    }

}