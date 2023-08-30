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

use std::sync::{Arc,Mutex,RwLock};
use std::path::{Path,PathBuf};
use crate::connection::connection::Connection;
use crate::connection::command::{CommandResult,cmd_info};
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskStatus, TaskResponse};
use crate::inventory::hosts::{Host,HostOSType};
use crate::playbooks::traversal::RunState;
use crate::tasks::fields::Field;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::tasks::FileAttributesEvaluated;
use crate::tasks::cmd_library::{screen_path,screen_general_input_strict,screen_general_input_loose};

pub struct Local {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>, 
    handle: Arc<Option<TaskHandle>>,
}

impl Local {

    pub fn new(run_state_handle: Arc<RunState>, connection_handle: Arc<Mutex<dyn Connection>>, host_handle: Arc<RwLock<Host>>,task_handle: Arc<TaskHandle>) -> Self {
        Self {
            run_state: run_state_handle,
            connection: connection_handle,
            host: host_handle,
            task_handle: Arc::new(None),

        }
    }

    pub fn attach_handle(task_handle: Arc<TaskHandle>) {
        self.handle = Some(task_handle);
    }

    fn unwrap_string_result(&self, request: &Arc<TaskRequest>, str_result: &Result<String,String>) -> Result<String, Arc<TaskResponse>> {
        return match str_result {
            Ok(x) => Ok(x.clone()),
            Err(y) => Err(self.is_failed(request, &y.clone()))
        };
    }

    fn run(&self, request: &Arc<TaskRequest>, cmd: &String, check_rc: CheckRc) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        
        assert!(request.request_type == TaskRequestType::Query, "local commands can only be run in query stage (was: {:?})", request.request_type);
        let response = self.handle.unwrap().response;
        // apply basic screening of the entire shell command, more filtering should already be done by cmd_library
        match screen_general_input_loose(&cmd) {
            Ok(x) => {},
            Err(y) => return Err(self.is_failed(request, &y.clone()))
        }
        let ctx = &self.run_state.context;
        let local_result = self.run_state.connection_factory.read().unwrap().get_local_connection(&ctx);
        let local_conn = match local_result {
            Ok(x) => x,
            Err(y) => { return response.is_failed(request, &y.clone())) }
        };
        let result = local_conn.lock().unwrap().run_command(self, request, cmd);

        if check_rc == CheckRc::Checked {
            if result.is_ok() {
                let ok_result = result.as_ref().unwrap();
                let cmd_result = ok_result.command_result.as_ref().as_ref().unwrap();
                if cmd_result.rc != 0 {
                    return response.command_failed(request, &Arc::new(Some(cmd_result.clone())));
                }
            }
        }
        return result;
    }

    pub fn read_file(&self, request: &Arc<TaskRequest>, path: &Path) -> Result<String, Arc<TaskResponse>> {
        return match crate::util::io::read_local_file(path) {
            Ok(s) => Ok(s),
            Err(x) => Err(self.is_failed(request, &x.clone()))
        };
    }

    fn internal_sha512(&self, request: &Arc<TaskRequest>, path: &String) -> Result<String,Arc<TaskResponse>> {
        let response = self.handle.unwrap().response;
        let localhost = self.get_localhost();
        let os_type = localhost.read().unwrap().os_type.expect("unable to detect host OS type");
        let get_cmd_result = crate::tasks::cmd_library::get_sha512_command(os_type, path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        let result = self.run(request, &cmd, CheckRc::Unchecked)?
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
                return response.is_failed(request, &format!("checksum failed: {}. {}", path, out));
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