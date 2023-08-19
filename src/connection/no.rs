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
// ABOUT: no.rs
// this is a dummy connection type that doesn't do anything, while it sounds useless
// we do use these connecton types when validating playbook language. It ensures
// that tasks have zero way of running, and is mostly here for provable correctness
// ===================================================================================

use crate::connection::connection::{Connection};
use crate::connection::factory::ConnectionFactory;
use crate::playbooks::context::PlaybookContext;
use crate::inventory::hosts::Host;
use crate::tasks::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub struct NoFactory {}

impl NoFactory { 
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionFactory for NoFactory {
    fn get_connection(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>,String> {
        return Ok(Arc::new(Mutex::new(NoConnection::new())));
    }
}

pub struct NoConnection {
}

impl NoConnection {
    pub fn new() -> Self {
        Self { }
    }
}

impl Connection for NoConnection {

   fn connect(&mut self) -> Result<(),String> {
       return Ok(());
   }

   fn run_command(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
       panic!("NoConnection may only be used for syntax checking");
   }

   // FIXME: this signature will change
   fn put_file(&self, data: String, remote_path: String, mode: Option<i32>) {
       panic!("NoConnection may only be used for syntax checking");
   }
}