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

use std::sync::{Arc,RwLock};
use std::path::Path;
use crate::connection::command::cmd_info;
use crate::tasks::{TaskRequest,TaskRequestType,TaskResponse};
use crate::inventory::hosts::Host;
use crate::playbooks::traversal::RunState;
use crate::tasks::cmd_library::screen_general_input_loose;
use crate::handle::handle::CheckRc;
use crate::handle::response::Response;

// local contains code that always executes on the control machine, whether in SSH mode or 'local' execution
// mode. The code that refers to the machine being configured is always in 'remote.rs', whether in SSH
// mode or using a local connection also!

pub struct Local {
    run_state: Arc<RunState>, 
    _host: Arc<RwLock<Host>>, 
    response: Arc<Response>,
}

impl Local {

    pub fn new(run_state_handle: Arc<RunState>, host_handle: Arc<RwLock<Host>>, response:Arc<Response>) -> Self {
        Self {
            run_state: run_state_handle,
            _host: host_handle,
            response: response
        }
    }

    pub fn get_localhost(&self) -> Arc<RwLock<Host>> {
        let inventory = self.run_state.inventory.read().unwrap();
        return inventory.get_host(&String::from("localhost"));
    }

    fn unwrap_string_result(&self, request: &Arc<TaskRequest>, str_result: &Result<String,String>) -> Result<String, Arc<TaskResponse>> {
        return match str_result {
            Ok(x) => Ok(x.clone()),
            Err(y) => Err(self.response.is_failed(request, &y.clone()))
        };
    }

    // runs a shell command.  These can only be executed in the query stage as we don't want anything done to actually configure
    // a machine in local.rs. 

    fn run(&self, request: &Arc<TaskRequest>, cmd: &String, check_rc: CheckRc) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        
        assert!(request.request_type == TaskRequestType::Query, "local commands can only be run in query stage (was: {:?})", request.request_type);
        // apply basic screening of the entire shell command, more filtering should already be done by cmd_library
        match screen_general_input_loose(&cmd) {
            Ok(_x) => {},
            Err(y) => return Err(self.response.is_failed(request, &y.clone()))
        }
        let ctx = &self.run_state.context;
        let local_result = self.run_state.connection_factory.read().unwrap().get_local_connection(&ctx);
        let local_conn = match local_result {
            Ok(x) => x,
            Err(y) => { return Err(self.response.is_failed(request, &y.clone())) }
        };
        let result = local_conn.lock().unwrap().run_command(&self.response, request, cmd);

        if check_rc == CheckRc::Checked {
            if result.is_ok() {
                let ok_result = result.as_ref().unwrap();
                let cmd_result = ok_result.command_result.as_ref().as_ref().unwrap();
                if cmd_result.rc != 0 {
                    return Err(self.response.command_failed(request, &Arc::new(Some(cmd_result.clone()))));
                }
            }
        }
        return result;
    }

    pub fn read_file(&self, request: &Arc<TaskRequest>, path: &Path) -> Result<String, Arc<TaskResponse>> {
        return match crate::util::io::read_local_file(path) {
            Ok(s) => Ok(s),
            Err(x) => Err(self.response.is_failed(request, &x.clone()))
        };
    }

    fn internal_sha512(&self, request: &Arc<TaskRequest>, path: &String) -> Result<String,Arc<TaskResponse>> {
        let localhost = self.get_localhost();
        let os_type = localhost.read().unwrap().os_type.expect("unable to detect host OS type");
        let get_cmd_result = crate::tasks::cmd_library::get_sha512_command(os_type, path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        let result = self.run(request, &cmd, CheckRc::Unchecked)?;
        let (rc, out) = cmd_info(&result);
        match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => {
                let value = out.split_whitespace().nth(0).unwrap().to_string();
                return Ok(value);
            },
            127 => {
                // file not found
                return Ok(String::from(""))
            },
            _ => {
                return Err(self.response.is_failed(request, &format!("checksum failed: {}. {}", path, out)));
            }
        };
    }

    pub fn get_sha512(&self, request: &Arc<TaskRequest>, path: &Path, use_cache: bool) -> Result<String,Arc<TaskResponse>> {
        let path2 = format!("{}", path.display());
        let localhost = self.get_localhost();
        if use_cache {
            let ctx = self.run_state.context.read().unwrap();
            let task_id = ctx.get_task_count();
            let mut localhost2 = localhost.write().unwrap();
            let cached = localhost2.get_checksum_cache(task_id, &path2);
            if cached.is_some() {
                return Ok(cached.unwrap());
            }
        }

        // this is a little weird.
        let value = self.internal_sha512(request, &path2)?;
        if use_cache {
            let mut localhost2 = localhost.write().unwrap();
            localhost2.set_checksum_cache(&path2, &value);
        }
        return Ok(value);
    }


}