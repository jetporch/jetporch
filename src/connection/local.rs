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

// ===================================================================================
// ABOUT: local.rs
// the local connection factory always returns a local connection for a host
// and is used by 'local' and 'check-local' CLI invocations.
// ===================================================================================

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

pub struct LocalFactory {}

impl LocalFactory { 
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionFactory for LocalFactory {
    fn get_connection(&self, _context: &Arc<RwLock<PlaybookContext>>, _host: &Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>,String> {
        return Ok(Arc::new(Mutex::new(LocalConnection::new())));
    }
}


pub struct LocalConnection {
}

impl LocalConnection {
    pub fn new() -> Self {
        Self { }
    }
}

impl Connection for LocalConnection {

    fn connect(&mut self) -> Result<(),String> {
        return Ok(());
    }

    fn run_command(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        // FIXME: eventually also add sudo strings in here
        let mut base = Command::new("sh");
        let command = base.arg("-c").arg(cmd).arg("2>&1");
        return match command.output() {
            Ok(x) => match x.status.code() {
                Some(0)      => Ok(handle.command_ok(request, CommandResult { out: convert_out(&x.stdout), rc: 0 })),
                Some(status) => Err(handle.command_failed(request, CommandResult { out: convert_out(&x.stdout), rc: status })),
                _            => Err(handle.command_failed(request, CommandResult { out: String::from(""), rc: 418 }))
            },
            Err(_x) => Err(handle.command_failed(request, CommandResult { out: String::from(""), rc: 404 }))
        }
    }

    // FIXME: this signature will change
    // FIXME: should return some type of result object
    fn put_file(&self, _data: String, _remote_path: String, _mode: Option<i32>) {
    }

}

// the input type is NOT string
fn convert_out(output: &Vec<u8>) -> String {
    return match std::str::from_utf8(output) {
        Ok(val) => val.to_string(),
        Err(_) => String::from("invalid UTF-8 characters in response"),
    };
}