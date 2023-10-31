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
use crate::handle::template::BlendTarget;
use serde::Deserialize;
use std::sync::Arc;

const MODULE: &str = "debug";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct DebugTask {
    pub name: Option<String>,
    pub vars: Option<Vec<String>>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

#[allow(dead_code)]
struct DebugAction {
    pub name: String,
    pub vars: Option<Vec<String>>,
}

impl IsTask for DebugTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(DebugAction {
                    name: self.name.clone().unwrap_or(String::from(MODULE)),
                    vars: self.vars.clone()
                }),
                with: Arc::new(PreLogicInput::template(handle, request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, tm, &self.and)?),
            }
        );
    }
}

impl IsAction for DebugAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                let mut map : serde_yaml::Mapping = serde_yaml::Mapping::new();
                let no_vars = self.vars.is_none();
                let blended = handle.run_state.context.read().unwrap().get_complete_blended_variables(&handle.host, BlendTarget::NotTemplateModule);
                for (k,v) in blended.iter() {
                    let k2 : String = match k {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => { panic!("invalid key in mapping"); }
                    };
                    if no_vars || self.vars.as_ref().unwrap().contains(&k2) {
                        if ! k2.eq(&String::from("item")) {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                }
                let msg = serde_yaml::to_string(&map).unwrap();
                let msg2 = format!("\n{}\n", msg);
                handle.debug(request, &msg2);
                return Ok(handle.response.is_passive(request));
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }

    }

}