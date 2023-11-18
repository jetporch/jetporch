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
use serde::{Deserialize};
use std::sync::{Arc,RwLock};
use serde_json;
use std::path::PathBuf;
use crate::inventory::hosts::Host;

const MODULE: &str = "External";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct ExternalTask {
    pub name: Option<String>,
    #[serde(rename = "use")]
    pub use_module: String,
    pub params: serde_json::Map<String, serde_json::Value>,
    pub save: Option<String>, 
    pub failed_when: Option<String>, 
    pub changed_when: Option<String>, 
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>,
}
struct ExternalAction {
    pub use_module: PathBuf,
    pub params: serde_json::Map<String, serde_json::Value>,
    pub save: Option<String>, 
    pub failed_when: Option<String>,
    pub changed_when: Option<String>,
}


impl IsTask for ExternalTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(ExternalAction {
                    use_module: handle.template.find_module_path(request, tm, &String::from("use"), &self.use_module)?,
                    // FIXME: template the parameters
                    params: {
                        self.params.clone()
                    },
                    save: handle.template.string_option_no_spaces(&request, tm, &String::from("save"), &self.save)?,
                    failed_when: handle.template.string_option_unsafe_for_shell(&request, tm, &String::from("failed_when"), &self.failed_when)?,
                    changed_when: handle.template.string_option_unsafe_for_shell(&request, tm, &String::from("changed_when"), &self.changed_when)?,
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for ExternalAction {
    
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_execution(&request));
            },

            TaskRequestType::Execute => {

                let (_tmp_path1, tmp_file1) = handle.remote.get_transfer_location(request)?;
                let (_tmp_path2, tmp_file2) = handle.remote.get_transfer_location(request)?;

                let module_tmp_file = tmp_file1.as_ref().unwrap();
                let param_tmp_file = tmp_file2.as_ref().unwrap();
                let module_str_path = module_tmp_file.as_path().display().to_string();
                let param_str_path = param_tmp_file.as_path().display().to_string();

                handle.remote.copy_file(request, self.use_module.as_path(), &module_str_path.clone(), |_f| { 
                    return Ok(()) 
                })?;
                
                let params_data = match serde_json::to_string(&self.params) {
                    Ok(x) => x,
                    Err(_y) => {
                        return Err(handle.response.is_failed(request,  &String::from("unable to load JSON inputs")));
                    }
                };
                handle.remote.write_data(request, &params_data, &param_str_path.clone(), |_f| {
                    // not using the after save handler for this module
                    return Ok(());
                })?;
                
                let chmod = format!("chmod +x '{}'", module_str_path.clone());
                handle.remote.run(request, &chmod, CheckRc::Checked)?;

                // FIXME: run the module, record the result
                let module_run = format!("{} < {}", module_str_path.clone(), param_str_path.clone());
                let task_result = handle.remote.run_unsafe(request, &module_run, CheckRc::Checked)?;
                let (rc, out) = cmd_info(&task_result);

                handle.remote.delete_file(request, &param_str_path.clone())?;
                handle.remote.delete_file(request, &module_str_path.clone())?;

                let map_data = build_results_map(handle, request, rc, &out)?;

                let should_fail = match self.failed_when.is_none() {
                    true => match rc { 0 => false, _ => true },
                    false => {
                        let condition = self.failed_when.as_ref().unwrap();
                        handle.template.test_condition_with_extra_data(request, TemplateMode::Strict, condition, &handle.host, map_data.clone())?
                    }
                };

                let should_mark_changed = match self.changed_when.is_none() {
                    true => true,
                    false => {
                        let condition = self.changed_when.as_ref().unwrap();
                        handle.template.test_condition_with_extra_data(request, TemplateMode::Strict, condition, &handle.host, map_data.clone())?
                    }
                };

                if self.save.is_some() {
                    save_results(&handle.host, self.save.as_ref().unwrap(), map_data);
                }

                return match should_fail {
                    true => Err(handle.response.command_failed(request, &Arc::clone(&task_result.command_result))),
                    false => match should_mark_changed {
                        true => Ok(task_result),
                        false => Ok(handle.response.is_passive(request))
                    }
                };
                
            },
    
            _ => { return Err(handle.response.not_supported(&request)); }
    
        }
    }

}

fn build_results_map(handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, rc: i32, out: &String) -> Result<serde_yaml::Mapping, Arc<TaskResponse>> {
    let mut result = serde_yaml::Mapping::new();
    let data : serde_yaml::Value = match serde_yaml::from_str(out) {
        Ok(x) => x,
        Err(_y) => { return Err(handle.response.is_failed(request, &format!("failed to parse external module response: {}", out))) },
    };
    result.insert(serde_yaml::Value::String(String::from("rc")), rc.into());
    match data {
        serde_yaml::Value::Mapping(data_map) => {
            for (k,v) in data_map.iter() {
                result.insert(k.clone(), v.clone());
            }
        },
        _ => {
            return Err(handle.response.is_failed(request, &format!("response should be a map: {}", out)));
        }
    }
    return Ok(result);
}

fn save_results(host: &Arc<RwLock<Host>>, key: &String, map_data: serde_yaml::Mapping) {
    let mut result = serde_yaml::Mapping::new();
    result.insert(serde_yaml::Value::String(key.clone()), serde_yaml::Value::Mapping(map_data.clone()));
    host.write().unwrap().update_variables(result);
}