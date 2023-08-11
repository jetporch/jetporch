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

use crate::connection::connection::{Connection,ConnectionCommandResult};
use crate::connection::factory::ConnectionFactory;
use crate::playbooks::context::PlaybookContext;

pub struct LocalFactory {}

impl LocalFactory { 
    pub fn new() -> Self {
        Self {}
    }
}

impl LocalFactory for ConnectionFactory {
    fn get_connection(context: &PlaybookContext, host: String) -> dyn Connection {
        return LocalConnection::new();
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

   fn connect(&mut self) {
   }

   fn run_command(&self, command: String) -> ConnectionCommandResult {
       ConnectionCommandResult {
           data: String::from("unimplemented"),
           exit_status: 0
       }
   }

   // FIXME: should return some type of result object
   fn put_file(&self, data: String, remote_path: String, mode: Option<i32>) {
   }

}
