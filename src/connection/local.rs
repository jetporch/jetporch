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

use crate::connection::connection::Connection;
use crate::connection::command::CommandResult;
use crate::connection::factory::ConnectionFactory;
use crate::playbooks::context::PlaybookContext;
use crate::inventory::hosts::Host;
use crate::tasks::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::process::Command;
use crate::Inventory;
use crate::util::io::jet_file_open;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::env;

#[allow(dead_code)]
pub struct LocalFactory {
    local_connection: Arc<Mutex<dyn Connection>>,
    inventory: Arc<RwLock<Inventory>>
}

impl LocalFactory {
    pub fn new(inventory: &Arc<RwLock<Inventory>>) -> Self {
        let host = inventory.read().expect("inventory read").get_host(&String::from("localhost"));
        let mut lc = LocalConnection::new(&Arc::clone(&host));
        lc.connect().expect("connection ok");
        Self {
            inventory: Arc::clone(&inventory),
            local_connection: Arc::new(Mutex::new(lc))
        }
    }
}
impl ConnectionFactory for LocalFactory {
    fn get_connection(&self, _context: &Arc<RwLock<PlaybookContext>>, _host: &Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>,String> {
        let conn : Arc<Mutex<dyn Connection>> = Arc::clone(&self.local_connection);
        return Ok(conn);
    }
    fn get_local_connection(&self, _context: &Arc<RwLock<PlaybookContext>>) -> Result<Arc<Mutex<dyn Connection>>, String> {
        let conn : Arc<Mutex<dyn Connection>> = Arc::clone(&self.local_connection);
        return Ok(conn);
    }

}

pub struct LocalConnection {
    host: Arc<RwLock<Host>>
}

impl LocalConnection {
    pub fn new(host: &Arc<RwLock<Host>>) -> Self {
        Self { host: Arc::clone(&host) }
    }
}

impl Connection for LocalConnection {

    fn whoami(&self) -> Result<String,String> {
        let user_result = env::var("USER");
        return match user_result {
            Ok(x) => Ok(x),
            Err(y) => Err(String::from("environment variable $USER: {y}"))
        };
    }

    fn connect(&mut self) -> Result<(),String> {

        let result = detect_os(&self.host);
        if result.is_ok() {
            return Ok(());
        }
        else {
            let (rc, out) = result.unwrap_err();
            return Err(out);
        }
    }

    fn run_command(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        // FIXME: eventually also add sudo strings in here
        let mut base = Command::new("sh");
        let command = base.arg("-c").arg(cmd).arg("2>&1");
        return match command.output() {
            Ok(x) => {
                match x.status.code() {
                    Some(rc) => {
                        let out = convert_out(&x.stdout,&x.stderr);
                        Ok(handle.command_ok(request,&Arc::new(Some(CommandResult { cmd: cmd.clone(), out: out.clone(), rc: rc }))))
                    },
                    None => {
                        Err(handle.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 418 }))))
                    }
                }
            },
            Err(_x) => {
                Err(handle.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 404 }))))
            }
        };
    }

    fn copy_file(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, src: &Path, remote_path: &String, mode: Option<i32>) -> Result<(), Arc<TaskResponse>> {
        // FIXME: this (temporary) implementation currently loads the file contents into memory which we do not want
        // copy the files with system calls instead.
        let remote_path2 = Path::new(remote_path);
        let result = std::fs::copy(src, &remote_path2);
        return match result {
            Ok(x) => Ok(()),
            Err(e) => { return Err(handle.is_failed(&request, &format!("copy failed: {:?}", e))) }
        }
    }

    fn write_data(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, data: &String, remote_path: &String, _mode: Option<i32>) -> Result<(),Arc<TaskResponse>> {
        let path = Path::new(&remote_path);
        if path.exists() {
            let mut file = match jet_file_open(path) {
                Ok(x) => x,
                Err(y) => return Err(handle.is_failed(&request, &format!("failed to open: {}: {:?}", remote_path, y)))
            };
            let write_result = write!(file, "{}", data);
            match write_result {
                Ok(_) => {},
                Err(y) => return Err(handle.is_failed(&request, &format!("failed to write: {}: {:?}", remote_path, y)))
            };
        } else {
            let mut file = match File::create(&path) {
                Ok(x) => x,
                Err(y) => return Err(handle.is_failed(&request, &format!("failed to create: {}: {:?}", remote_path, y)))
            };
            let write_result = write!(file, "{}", data);
            match write_result {
                Ok(_) => {},
                Err(y) => return Err(handle.is_failed(&request, &format!("failed to write: {}: {:?}", remote_path, y)))
            };
        }
        return Ok(());
    }

}

fn convert_out(output: &Vec<u8>, err: &Vec<u8>) -> String {
    let mut base = match std::str::from_utf8(output) {
        Ok(val) => val.to_string(),
        Err(_) => String::from("invalid UTF-8 characters in response"),
    };
    let rest = match std::str::from_utf8(err) {
        Ok(val) => val.to_string(),
        Err(_) => String::from("invalid UTF-8 characters in response"),
    };
    base.push_str("\n");
    base.push_str(&rest);
    return base.trim().to_string();

}

// connection runs uname -a on connect to check the OS type.
fn detect_os(host: &Arc<RwLock<Host>>) -> Result<(),(i32, String)> {
    let mut base = Command::new("uname");
    let command = base.arg("-a");
    return match command.output() {
        Ok(x) => match x.status.code() {
            Some(0)      => {
                let out = convert_out(&x.stdout,&x.stderr);
                {
                    match host.write().unwrap().set_os_info(&out) {
                        Ok(_) => { },
                        Err(_) => { return Err((500, String::from("failed to set OS info"))); }
                    }
                }
                Ok(())
            }
            Some(status) => Err((status, convert_out(&x.stdout, &x.stderr))),
            _            => Err((418, String::from("uname -a failed without status code")))
        },
        Err(_x) => Err((418, String::from("uname -a failed without status code")))
    }
}
