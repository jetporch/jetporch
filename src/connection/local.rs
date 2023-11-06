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
use crate::playbooks::context::PlaybookContext;
use crate::connection::factory::ConnectionFactory;
use crate::connection::command::Forward;

use crate::inventory::hosts::Host;
use crate::handle::response::Response;
use crate::tasks::{TaskRequest,TaskResponse};

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

// implementation for both the local connection factory and local connections

#[allow(dead_code)]
pub struct LocalFactory {
    local_connection: Arc<Mutex<dyn Connection>>,
    inventory: Arc<RwLock<Inventory>>
}

impl LocalFactory {
    pub fn new(inventory: &Arc<RwLock<Inventory>>) -> Self {

        // we require a localhost to be in the inventory and immediately construct a connection to it

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
        // rather than producing new connections, this always returns a clone of the already established local connection from the constructor
        let conn : Arc<Mutex<dyn Connection>> = Arc::clone(&self.local_connection);
        return Ok(conn);
    }
    fn get_local_connection(&self, _context: &Arc<RwLock<PlaybookContext>>) -> Result<Arc<Mutex<dyn Connection>>, String> {
        let conn : Arc<Mutex<dyn Connection>> = Arc::clone(&self.local_connection);
        return Ok(conn);
    }

}

pub struct LocalConnection {
    host: Arc<RwLock<Host>>,
}

impl LocalConnection {
    pub fn new(host: &Arc<RwLock<Host>>) -> Self {
        Self { host: Arc::clone(&host) }
    }

    fn trim_newlines(&self, s: &mut String) {
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
    }
}

impl Connection for LocalConnection {

    fn whoami(&self) -> Result<String,String> {
        // get the currently logged in user.
        let user_result = env::var("USER");
        return match user_result {
            Ok(x) => Ok(x),
            Err(y) => Err(format!("environment variable $USER: {y}"))
        };
    }

    fn connect(&mut self) -> Result<(),String> {
        // upon connection make sure the localhost detection routine runs
        let result = detect_os(&self.host);
        if result.is_ok() {
            return Ok(());
        }
        else {
            let (_rc, out) = result.unwrap_err();
            return Err(out);
        }
    }

    fn run_command(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, cmd: &String, _forward: Forward) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let mut base = Command::new("sh");
        let cmd2 = format!("LANG=C {}", cmd);
        let command = base.arg("-c").arg(cmd2).arg("2>&1");
        match command.output() {
            Ok(x) => {
                match x.status.code() {
                    Some(rc) => {
                        let mut out = convert_out(&x.stdout,&x.stderr);
                        self.trim_newlines(&mut out);
                        return Ok(response.command_ok(request,&Arc::new(Some(CommandResult { cmd: cmd.clone(), out: out.clone(), rc: rc }))));
                    },
                    None => {
                        return Err(response.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 418 }))));
                    }
                }
            },
            Err(_x) => {
                return Err(response.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 404 }))));
            }
        };
    }

    fn copy_file(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, src: &Path, remote_path: &String) -> Result<(), Arc<TaskResponse>> {
        // FIXME: this (temporary) implementation currently loads the file contents into memory which we do not want
        // copy the files with system calls instead.
        let remote_path2 = Path::new(remote_path);
        let result = std::fs::copy(src, &remote_path2);
        return match result {
            Ok(_x) => Ok(()),
            Err(e) => { return Err(response.is_failed(&request, &format!("copy failed: {:?}", e))) }
        }
    }

    fn write_data(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, data: &String, remote_path: &String) -> Result<(),Arc<TaskResponse>> {
        let path = Path::new(&remote_path);
        if path.exists() {
            let mut file = match jet_file_open(path) {
                Ok(x) => x,
                Err(y) => return Err(response.is_failed(&request, &format!("failed to open: {}: {:?}", remote_path, y)))
            };
            let write_result = write!(file, "{}", data);
            match write_result {
                Ok(_) => {},
                Err(y) => return Err(response.is_failed(&request, &format!("failed to write: {}: {:?}", remote_path, y)))
            };
        } else {
            let mut file = match File::create(&path) {
                Ok(x) => x,
                Err(y) => return Err(response.is_failed(&request, &format!("failed to create: {}: {:?}", remote_path, y)))
            };
            let write_result = write!(file, "{}", data);
            match write_result {
                Ok(_) => {},
                Err(y) => return Err(response.is_failed(&request, &format!("failed to write: {}: {:?}", remote_path, y)))
            };
        }
        return Ok(());
    }

}

pub fn convert_out(output: &Vec<u8>, err: &Vec<u8>) -> String {
    // output from the Rust command class can contain junk bytes, here we mostly don't try to solve this yet
    // and will basically fail if output contains junk. This may be dealt with later.
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

fn detect_os(host: &Arc<RwLock<Host>>) -> Result<(),(i32, String)> {
    // upon connection we run uname -a on connect to check the OS type.
    
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
