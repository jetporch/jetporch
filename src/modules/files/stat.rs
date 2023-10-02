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

use crate::{tasks::*, inventory::hosts::Host};
use crate::handle::handle::TaskHandle;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

const MODULE: &str = "stat";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct StatTask {
    pub name: Option<String>,
    pub path: String,
    pub save: String,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

#[allow(dead_code)]
struct StatAction {
    pub path: String,
    pub save: String,
}

impl IsTask for StatTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(StatAction {
                    path: handle.template.path(&request, tm, &String::from("path"), &self.path)?,
                    save: handle.template.string_no_spaces(&request, tm, &String::from("save"), &self.save)?,
                }),
                with: Arc::new(PreLogicInput::template(handle, request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, tm, &self.and)?),
            }
        );
    }
}

impl IsAction for StatAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                let stat = stat_file(handle, request, &self.path)?;

                match serde_yaml::to_value(stat) {
                    Ok(value) => save_results(&handle.host, &self.save, value),
                    Err(x) => {
                        return Err(handle.response.is_failed(request, &format!("failed serializing stat results: {}", x)));
                    },
                };

                Ok(handle.response.is_passive(request))
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }

    }

}

#[derive(Serialize)]
struct StatResult {
    pub exists: bool,
    pub is_dir: Option<bool>,
    pub mode: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
}

const DOESNT_EXIST: StatResult = StatResult{
    exists: false,
    is_dir: None,
    mode: None,
    owner: None,
    group: None,
};

fn stat_file(handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, path: &String) -> Result<StatResult, Arc<TaskResponse>> {
    let mode_option = handle.remote.get_mode(request, path)?;
    match mode_option {
        Some(mode) => {
            let is_dir = handle.remote.get_is_directory(request, path)?;
            let ownership_option = handle.remote.get_ownership(request, path)?;
            match ownership_option {
                Some((owner, group)) => Ok(StatResult{
                    exists: is_dir,
                    is_dir: Some(true),
                    mode: Some(mode),
                    owner: Some(owner),
                    group: Some(group),
                }),
                None => Ok(DOESNT_EXIST),
            }
        },
        None => Ok(DOESNT_EXIST),
    }
}

fn save_results(host: &Arc<RwLock<Host>>, key: &String, value: serde_yaml::Value) {
    let mut result = serde_yaml::Mapping::new();
    result.insert(serde_yaml::Value::String(key.clone()), value);
    host.write().unwrap().update_variables(result);
}