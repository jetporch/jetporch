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
//use std::sync::Arc;
//#[allow(unused_imports)]
use serde::{Deserialize};

#[derive(Deserialize,Debug)]
#[serde(tag="echo",deny_unknown_fields)]
pub struct Echo {
    pub name: Option<String>,
    pub msg: String,
    pub with: Option<PreLogic>,
    pub and: Option<PostLogic>
}

impl Echo {
    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Echo, Arc<TaskResponse>> {
        return Ok(Echo {
            name: self.name.clone(),
            msg: handle.template(&request, &self.msg)?,
            with: PreLogic::template(&handle, &request, &self.with)?,
            and: PostLogic::template(&handle, &request, &self.and)?
        });
    }
}

impl IsTask for Echo {

    fn get_module(&self) -> String { String::from("Echo") }
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
