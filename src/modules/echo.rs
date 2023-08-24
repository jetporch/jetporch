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
//#[allow(unused_imports)]
use serde::{Deserialize};

const MODULE: &'static str = "Echo";

#[derive(Deserialize,Debug)]
#[serde(tag="echo",deny_unknown_fields)]
pub struct Echo {
    pub name: Option<String>,
    pub msg: String,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct Evaluated {
    pub name: String,
    pub msg: String,
    pub with: Option<PreLogicEvaluated>,
    pub and: Option<PostLogicEvaluated>
}

impl Echo {
    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Evaluated, Arc<TaskResponse>> {
        return Ok(Evaluated {
            name: self.name.clone().unwrap_or(String::from(MODULE)),
            msg:  handle.template_string(&request, &String::from("msg"), &self.msg)?,
            with: PreLogicInput::template(&handle, &request, &self.with)?,
            and:  PostLogicInput::template(&handle, &request, &self.and)?
        });
    }
}

impl IsTask for Echo {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Validate => {
                let evaluated = self.evaluate(handle, request)?;
                return Ok(handle.is_validated(&request, &Arc::new(evaluated.with), &Arc::new(evaluated.and)));
            },

            TaskRequestType::Query => {
                return Ok(handle.needs_passive(&request));
            },

            TaskRequestType::Passive => {
                let evaluated = self.evaluate(handle, request)?;
                handle.debug(&request, &evaluated.msg);
                return Ok(handle.is_passive(&request));
            },

            _ => { return Err(handle.not_supported(&request)); }
        
        }
    }

}
