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
use serde::{Deserialize};
use std::sync::{Arc};


const MODULE: &'static str = "Set";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct SetTask {
    pub name: Option<String>,
    pub vars: Option<serde_yaml::Mapping>, 
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>,

}
struct SetAction {
    pub vars: Option<serde_yaml::Mapping>, 
}


impl IsTask for SetTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(SetAction {
                    vars: self.vars.clone() /* templating will happen below */
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for SetAction {
    
    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(&request));
            },

            TaskRequestType::Passive => {
                
                /* so far this only templates top level strings, which is probably sufficient, rather than strings found in deeper levels */

                let mut mapping = serde_yaml::Mapping::new();
                if self.vars.as_ref().is_some() {
                    for (k,v) in self.vars.as_ref().unwrap().iter() {
                        if v.is_string() {
                            let ks = v.as_str().unwrap().to_string();
                            let vs = v.as_str().unwrap().to_string();
                            let templated = handle.template.string_unsafe_for_shell(request, &ks.clone(), &vs)?;
                            mapping.insert(k.clone(), serde_yaml::Value::String(templated));
                        } else {
                            mapping.insert(k.clone(), v.clone());
                        }   
                    }
                }

                handle.host.write().unwrap().update_variables(mapping);
                return Ok(handle.response.is_passive(&request));
            
            }

            _ => { return Err(handle.response.not_supported(request)); }

        }
    }

}

