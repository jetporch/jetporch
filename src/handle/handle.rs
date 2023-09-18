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
use crate::connection::connection::Connection;
use crate::tasks::request::TaskRequest;
use crate::inventory::hosts::Host;
use crate::playbooks::traversal::RunState;

use crate::handle::local::Local;
use crate::handle::remote::Remote;
use crate::handle::template::Template;
use crate::handle::response::Response;

// task handles are given to modules to give them shortcuts to work with the jet system
// actual functionality is mostly provided via TaskRequest/TaskResponse and such, the handles
// are mostly module authors don't need to think about how things work as much.  This is
// especially true for the finite state machine that executes tasks.

// whether commands should treat non-zero returns as errors
#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum CheckRc {
    Checked,
    Unchecked
}

pub struct TaskHandle {
    pub run_state: Arc<RunState>, 
    _connection: Arc<Mutex<dyn Connection>>,
    pub host: Arc<RwLock<Host>>,
    pub local: Arc<Local>,
    pub remote: Arc<Remote>,
    pub response: Arc<Response>,
    pub template: Arc<Template>,
}

impl TaskHandle {

    pub fn new(run_state_handle: Arc<RunState>, connection_handle: Arc<Mutex<dyn Connection>>, host_handle: Arc<RwLock<Host>>) -> Self {

        // since we can't really have back-references (thanks Rust?) we pass to each namespace what we need of the others
        // thankfully, no circular references seem to be required :)

        // response contains namespaced shortcuts for returning results from module calls
        let response = Arc::new(Response::new(
            Arc::clone(&run_state_handle), 
            Arc::clone(&host_handle)
        ));

        // template contains various functions around templating strings, and is most commonly seen in processing module
        // input parameters as well as directly used in modules like template. It's also used in a few places inside
        // the engine itself.
        let template = Arc::new(Template::new(
            Arc::clone(&run_state_handle), 
            Arc::clone(&host_handle),
            Arc::clone(&response)
        ));

        // remote contains code for interacting with the host being configured.  The host could actually be 'localhost', but it's usually
        // a machine different from the control machine.  this could be called "configuration_target" instead but that would be more typing
        let remote = Arc::new(Remote::new(
            Arc::clone(&run_state_handle), 
            Arc::clone(&connection_handle), 
            Arc::clone(&host_handle),
            Arc::clone(&template),
            Arc::clone(&response)
        ));

        // local contains code that is related to looking at the control machine.  Even in local configuration modes, functions here are
        // not used to configure the actual system, those would be from remote. this could be thought of as 'control-machine-side-module-support'.

        let local = Arc::new(Local::new(
            Arc::clone(&run_state_handle), 
            Arc::clone(&host_handle),
            Arc::clone(&response)
        ));

        // the handle itself allows access to all of the above namespaces and also has a reference to the host being configured.
        // run_state itself is a bit of a pseudo-global and contains quite a few more parameters, see playbook/traversal.rs for
        // what it contains. 

        return Self {
            run_state: Arc::clone(&run_state_handle),
            _connection: Arc::clone(&connection_handle),
            host: Arc::clone(&host_handle),
            remote: Arc::clone(&remote),
            local: Arc::clone(&local),
            response: Arc::clone(&response),
            template: Arc::clone(&template),
        };
    }

    #[inline(always)]
    pub fn debug(&self, _request: &Arc<TaskRequest>, message: &String) {
        self.run_state.visitor.read().unwrap().debug_host(&self.host, message);
    }

}