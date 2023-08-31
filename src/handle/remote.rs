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
use crate::handle::handle::{CheckRc,TaskHandle};
use crate::handle::template::Safety;
use crate::handle::response::Response;

pub struct Remote {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>, 
    response: Arc<Response>
}

impl Remote {

    pub fn new(run_state_handle: Arc<RunState>, connection_handle: Arc<Mutex<dyn Connection>>, host_handle: Arc<RwLock<Host>>, response: Arc<Response>) -> Self {
        Self {
            run_state: run_state_handle,
            connection: connection_handle,
            host: host_handle,
            response: response,
        }
    }

    fn unwrap_string_result(&self, request: &Arc<TaskRequest>, str_result: &Result<String,String>) -> Result<String, Arc<TaskResponse>> {
        return match str_result {
            Ok(x) => Ok(x.clone()),
            Err(y) => {
                return Err(self.response.is_failed(request, &y.clone()));
            }
        };
    }

    pub fn whoami(&self) -> Result<String,String> {
        return self.connection.lock().unwrap().whoami();
    }

    pub fn run(&self, request: &Arc<TaskRequest>, cmd: &String, check_rc: CheckRc) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        return self.internal_run(request, cmd, Safety::Safe, check_rc);
    }

    pub fn run_unsafe(&self, request: &Arc<TaskRequest>, cmd: &String, check_rc: CheckRc) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        return self.internal_run(request, cmd, Safety::Unsafe, check_rc);
    }

    fn internal_run(&self, request: &Arc<TaskRequest>, cmd: &String, safe: Safety, check_rc: CheckRc) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        assert!(request.request_type != TaskRequestType::Validate, "commands cannot be run in validate stage");
        // apply basic screening of the entire shell command, more filtering should already be done by cmd_library
        // for parameterized calls that use that
        if safe == Safety::Safe {
            match screen_general_input_loose(&cmd) {
                Ok(x) => {},
                Err(y) => return Err(self.response.is_failed(request, &y.clone()))
            }
        }
        let result = self.connection.lock().unwrap().run_command(&self.response, request, cmd);

        // FIXME: this is reused below, move into function (see run_local)
        if check_rc == CheckRc::Checked {
            if result.is_ok() {
                let ok_result = result.as_ref().unwrap();
                let cmd_result = ok_result.command_result.as_ref().as_ref().unwrap();
                if cmd_result.rc != 0 {
                    // FIXME: since cmd_result is cloneable there is no need for it to be in an Arc
                    return Err(self.response.command_failed(request, &Arc::new(Some(cmd_result.clone()))));
                }
            }
        }

        return result
    }

    pub fn get_os_type(&self) -> HostOSType {
        let os_type = self.host.read().unwrap().os_type;
        if os_type.is_none() {
            panic!("failed to detect OS type for {}, bailing out", self.host.read().unwrap().name);
        }
        return os_type.unwrap();
    }

    // writes a string (for example, from a template) to a remote file location
    pub fn write_data(&self, request: &Arc<TaskRequest>, data: &String, path: &String, mode: Option<i32>) -> Result<(), Arc<TaskResponse>> {
        return self.connection.lock().unwrap().write_data(&self.response, request, &data.clone(), &path.clone(), mode);
    }

    pub fn copy_file(&self, request: &Arc<TaskRequest>, src: &Path, dest: &String, mode: Option<i32>) -> Result<(), Arc<TaskResponse>> {
        let owner_result = self.get_ownership(request, dest)?;
        let (mut old_owner, mut old_group) = (String::from("root"), String::from("root"));
        let whoami : String;
        let mut flip_owner: bool = false;

        if owner_result.is_some() {
            // the file exists
            (old_owner, old_group) = owner_result.unwrap();
            let whoami = match self.whoami() {
                Ok(x) => x,
                Err(y) => { return Err(self.response.is_failed(request, &y.clone())) }
            };
            if ! old_owner.eq(&whoami) {
                flip_owner = true;
                self.set_owner(request, &dest, &whoami)?;
            }
        }
        self.connection.lock().unwrap().copy_file(&self.response, &request, src, &dest.clone(), mode)?;
        if flip_owner {
            self.set_owner(request, &dest, &old_owner)?;
        }
        return Ok(());
    }

    pub fn get_mode(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Option<String>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_mode_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        
        let result = self.run(request, &cmd, CheckRc::Unchecked)?;
        let (rc, out) = cmd_info(&result);
        return match rc {
            // we can all unwrap because all possible string lists will have at least 1 element
            0 => Ok(Some(out.split_whitespace().nth(0).unwrap().to_string())),
            _ => Ok(None),
        }
    }

    pub fn is_directory(&self, request: &Arc<TaskRequest>, path: &String) -> Result<bool,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_is_directory_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        
        let result = self.run(request, &cmd, CheckRc::Checked)?;
        let (rc, out) = cmd_info(&result);
        // so far this assumes reliable ls -ld output across all supported operating systems, this may change
        // in wich case we may need to consider os_type here
        if out.starts_with("d") {
            return Ok(true);
        }
        return Ok(false);
    }

    pub fn touch_file(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_touch_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        return self.run(request, &cmd, CheckRc::Checked);  
    }

    pub fn delete_file(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_delete_file_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        return self.run(request, &cmd, CheckRc::Checked);  
    }

    pub fn get_ownership(&self, request: &Arc<TaskRequest>, path: &String) -> Result<Option<(String,String)>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::get_ownership_command(self.get_os_type(), path);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        
        let result = self.run(request, &cmd, CheckRc::Unchecked)?;
        let (rc, out) = cmd_info(&result);

        match rc {
            0 => {},
            _ => { return Ok(None); },
        }

        let mut split = out.split_whitespace();
        let owner = match split.nth(2) {
            Some(x) => x,
            None => { 
                return Err(self.response.is_failed(request, &format!("unexpected output format from {}: {}", cmd, out)));
            }
        };
        // this is a progressive iterator, hence 0 and not 3 for nth() below!
        let group = match split.nth(0) {
            Some(x) => x,
            None => { 
                return Err(self.response.is_failed(request, &format!("unexpected output format from {}: {}", cmd, out))); 
            }
        };
        return Ok(Some((owner.to_string(),group.to_string())));
    }

    pub fn set_owner(&self, request: &Arc<TaskRequest>, remote_path: &String, owner: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::set_owner_command(self.get_os_type(), remote_path, owner);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        return self.run(request,&cmd,CheckRc::Checked);
    }

    pub fn set_group(&self, request: &Arc<TaskRequest>, remote_path: &String, group: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::set_group_command(self.get_os_type(), remote_path, group);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        return self.run(request,&cmd,CheckRc::Checked);
    }

    pub fn set_mode(&self, request: &Arc<TaskRequest>, remote_path: &String, mode: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let get_cmd_result = crate::tasks::cmd_library::set_mode_command(self.get_os_type(), remote_path, mode);
        let cmd = self.unwrap_string_result(&request, &get_cmd_result)?;
        return self.run(request,&cmd,CheckRc::Checked);
    }

    pub fn get_sha512(&self, request: &Arc<TaskRequest>, path: &String) -> Result<String,Arc<TaskResponse>> {
        // FIXME: move function code here and eliminate
        return self.internal_sha512(request, path);
    }
   

    fn internal_sha512(&self, request: &Arc<TaskRequest>, path: &String) -> Result<String,Arc<TaskResponse>> {
        
        // these games around local command execution should only happen here and not complicate other functions, 
        // local_action/delegate_to will happen at a higher level in the taskfsm.

        let os_type = self.get_os_type();
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


    pub fn query_common_file_attributes(&self, request: &Arc<TaskRequest>, remote_path: &String, 
        attributes_in: &Option<FileAttributesEvaluated>, changes: &mut Vec<Field>) -> Result<Option<String>,Arc<TaskResponse>> {
        let remote_mode = self.get_mode(request, remote_path)?;
        
        if remote_mode.is_none() {
            changes.push(Field::Content);
            return Ok(None);
        }
        if attributes_in.is_some() {
            let attributes = attributes_in.as_ref().unwrap();
            let owner_result = self.get_ownership(request, remote_path)?;
            if owner_result.is_none() {
                return Err(self.response.is_failed(request, &String::from("file was deleted unexpectedly mid-operation")));
            }
            let (remote_owner, remote_group) = owner_result.unwrap();

            if attributes.owner.is_some() {
                if ! remote_owner.eq(attributes.owner.as_ref().unwrap()) { 
                    changes.push(Field::Owner); 
                }
            }
            if attributes.group.is_some() {
                if ! remote_group.eq(attributes.group.as_ref().unwrap())  { 
                    changes.push(Field::Group); 
                }
            }
            if attributes.mode.is_some() {
                if ! remote_mode.as_ref().unwrap().eq(attributes.mode.as_ref().unwrap()) { 
                    changes.push(Field::Mode); 
                }
            }
        }
        return Ok(remote_mode);
    }

    pub fn process_common_file_attributes(&self, 
        request: &Arc<TaskRequest>, 
        remote_path: &String, 
        attributes_in: &Option<FileAttributesEvaluated>, 
        changes: &Vec<Field>)
            -> Result<(),Arc<TaskResponse>> {

        if attributes_in.is_none() {
            return Ok(());
        }
        let attributes = attributes_in.as_ref().unwrap();

        for change in changes.iter() {
            match change {
                Field::Owner => {
                    assert!(attributes.owner.is_some(), "owner is set");
                    self.set_owner(request, remote_path, &attributes.owner.as_ref().unwrap())?;
                },
                Field::Group => {
                    assert!(attributes.group.is_some(), "owner is set");
                    self.set_group(request, remote_path, &attributes.group.as_ref().unwrap())?;
                },
                Field::Mode => {
                    assert!(attributes.mode.is_some(), "owner is set");
                    self.set_mode(request, remote_path, &attributes.mode.as_ref().unwrap())?;
                },
                _ => {}
            }
        }
        return Ok(());
    }

    pub fn process_all_common_file_attributes(&self, 
        request: &Arc<TaskRequest>, 
        remote_path: &String, 
        attributes_in: &Option<FileAttributesEvaluated>) 
             -> Result<(),Arc<TaskResponse>> {

        let mut all = Field::all_file_attributes();
        return self.process_common_file_attributes(request, remote_path, attributes_in, &all);
    }


}