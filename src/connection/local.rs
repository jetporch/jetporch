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

use crate::connection::connection::{Connection};
use crate::connection::command::{CommandResult};
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

pub struct LocalFactory {
    local_connection: Arc<Mutex<dyn Connection>>,
    inventory: Arc<RwLock<Inventory>>
}

impl LocalFactory { 
    pub fn new(inventory: &Arc<RwLock<Inventory>>) -> Self { 
        let host = inventory.read().unwrap().get_host(&String::from("localhost"));
        Self {
            inventory: Arc::clone(&inventory),
            local_connection: Arc::new(Mutex::new(LocalConnection::new(&Arc::clone(&host))))
        } 
    }
}
impl ConnectionFactory for LocalFactory {
    fn get_connection(&self, _context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>,String> {
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

    fn connect(&mut self) -> Result<(),String> {
        detect_os(&self.host);
        return Ok(());
    }

    fn run_command(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        // FIXME: eventually also add sudo strings in here
        let mut base = Command::new("sh");
        let command = base.arg("-c").arg(cmd).arg("2>&1");
        return match command.output() {
            Ok(x) => match x.status.code() {
                Some(y)      => Ok(handle.command_ok(request, CommandResult { cmd: cmd.clone(), out: convert_out(&x.stdout), rc: y })),
                _            => Err(handle.command_failed(request, CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 418 }))
            },
            Err(_x) => Err(handle.command_failed(request, CommandResult { cmd: cmd.clone(), out: String::from(""), rc: 404 }))
        }
    }

    // FIXME: implement, this signature will change, should return some type of result object
    fn put_file(&self, _data: String, _remote_path: String, _mode: Option<i32>) {
    }

}

fn convert_out(output: &Vec<u8>) -> String {
    return match std::str::from_utf8(output) {
        Ok(val) => val.to_string(),
        Err(_) => String::from("invalid UTF-8 characters in response"),
    };
}

// connection runs uname -a on connect to check the OS type.
fn detect_os(host: &Arc<RwLock<Host>>) -> Result<(),(i32, String)> {
    let mut base = Command::new("uname");
    let command = base.arg("-a");
    return match command.output() {
        Ok(x) => match x.status.code() {
            Some(0)      => { 
                let out = convert_out(&x.stdout);
                {
                    println!("LOCAL LOCK!");
                    match host.write().unwrap().set_os_info(&out) {
                        Ok(_) => {},
                        Err(_) => { return Err((500, String::from("failed to set OS info"))); }
                    }
                    println!("LOCAL LOCK CLEAR!");
                }
                Ok(())
            }
            Some(status) => Err((status, convert_out(&x.stdout))),
            _            => Err((418, String::from("uname -a failed without status code")))
        },
        Err(_x) => Err((418, String::from("uname -a failed without status code")))
    }
}