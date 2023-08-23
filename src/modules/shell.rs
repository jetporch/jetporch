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

#[derive(Deserialize,Debug)]
#[serde(tag="shell",deny_unknown_fields)]
pub struct Shell {
    pub name: Option<String>,
    pub cmd: String,
    pub verbose: Option<bool>,
    pub with: Option<PreLogic>,
    pub and: Option<PostLogic>
}

impl Shell {
    pub fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Shell, Arc<TaskResponse>> {
        return Ok(Shell {
            name: self.name.clone(),
            cmd: handle.template(&request, &self.cmd)?,
            verbose: self.verbose,
            with: PreLogic::template(&handle, &request, &self.with)?,
            and: PostLogic::template(&handle, &request, &self.and)?
        });
    }
}

impl IsTask for Shell {

    fn get_module(&self) -> String { String::from("Shell") }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Validate => {
                let evaluated = self.evaluate(handle, request)?;
                return Ok(handle.is_validated(&request, &Arc::new(evaluated.with), &Arc::new(evaluated.and)));
            },

            TaskRequestType::Query => {
                return Ok(handle.needs_execution(&request));
            },
    
            TaskRequestType::Execute => {
                let evaluated = self.evaluate(handle, request)?;
                let result = handle.run(&request, &evaluated.cmd.clone());
                let (rc, out) = cmd_info(&result);

                // FIXME: verbosity should be passed to handle.run, then embed this logic there
                // methods in handle should be able to check verbosity levels.
                if self.verbose.is_some() && self.verbose.unwrap() {
                    let display = vec![ format!("rc: {}",rc), format!("out: {}", out) ];
                    handle.debug_lines(&request, &display);
                }

                return result;
            },
    
            _ => { return Err(handle.not_supported(&request)); }
    
        }
    }

}
