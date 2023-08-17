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
// ABOUT: task_fsm.rs
// the task FSM is mostly about proveable correctness in executing modules - ensuring
// that modules are implemented to be able to ask what they want to do, and to make
// sure they respond correctly. It will also be used to loop over hosts in parallel
// particularly needed for SSH modes of the program
// ===================================================================================

use crate::tasks::response::{TaskStatus,TaskResponse};
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;
use crate::connection::factory::ConnectionFactory;
use crate::connection::no::NoFactory;
use crate::registry::list::Task;
use crate::connection::connection::Connection;
use crate::tasks::handle::TaskHandle;

use crate::tasks::request::TaskRequest;
use crate::inventory::inventory::Inventory;
use crate::inventory::hosts::Host;
use std::sync::{Arc,Mutex,RwLock};

// run a task on one or more hosts -- check modes (syntax/normal), or for 'real', on any connection type

pub fn fsm_run_task(
    inventory: &Arc<RwLock<Inventory>>, 
    context: &Arc<RwLock<PlaybookContext>>,
    visitor: &Arc<RwLock<dyn PlaybookVisitor>>, 
    connection_factory: &Arc<RwLock<dyn ConnectionFactory>>, 
    task: &Task) -> Result<(), String> {

    let tmp_localhost = Arc::new(RwLock::new(Host::new(&String::from("localhost"))));
    let no_connection = NoFactory::new().get_connection(context, &String::from("localhost")).unwrap();

    let syntax_check_result = run_task_on_host(
        inventory, 
        context, 
        visitor, 
        &no_connection, 
        &tmp_localhost, 
        task
    );

    match syntax_check_result.status {
        TaskStatus::IsValidated => { 
            if visitor.read().unwrap().is_syntax_only() { return Ok(()); }
        }, 
        TaskStatus::Failed => { 
            return Err(format!("parameters conflict: {}", syntax_check_result.msg.as_ref().unwrap()));
        },
        _ => { panic!("module returned invalid response to syntax check") }
    }

    let hosts = context.read().unwrap().get_all_hosts();
    for host in hosts {
        let connection_result = connection_factory.read().unwrap().get_connection(context, &host.clone());
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                let host_object = inventory.read().unwrap().get_host(&host);

                let task_response = run_task_on_host(
                    inventory, 
                    context, 
                    visitor, 
                    &connection, 
                    &host_object, 
                    task
                );

                if task_response.is_failed() {
                    // FIXME: visitor does not need locks around it!
                    visitor.read().unwrap().on_host_task_failed(context, &task_response, host.clone());
                }
            },
            Err(_) => { 
                visitor.read().unwrap().on_host_connect_failed(context, host.clone());
            }
        }
    }
    return Ok(());
}


// the "on this host" method body from fsm_run_task

fn run_task_on_host(
    inventory: &Arc<RwLock<Inventory>>, 
    context: &Arc<RwLock<PlaybookContext>>,
    visitor: &Arc<RwLock<dyn PlaybookVisitor>>, 
    connection: &Arc<Mutex<dyn Connection>>,
    host: &Arc<RwLock<Host>>, 
    task: &Task) -> Arc<TaskResponse> {

    let syntax      = visitor.read().unwrap().is_syntax_only();
    let modify_mode = ! visitor.read().unwrap().is_check_mode();

    let handle = Arc::new(TaskHandle::new(Arc::clone(inventory), 
        Arc::clone(context), 
        Arc::clone(visitor), 
        Arc::clone(connection), 
        Arc::clone(host))
    );

    let task_ptr = Arc::new(task);

    let vrc = task.dispatch(&handle, &TaskRequest::validate());
    match vrc.status {
        TaskStatus::IsValidated => { 
            if syntax {
                return vrc;
            }
        },
        TaskStatus::Failed => { return vrc; },
        _ => { panic!("module internal fsm state invalid (on verify)") }
    }


    let query = TaskRequest::query();
    let qrc = task.dispatch(&handle, &TaskRequest::query());
    let result = match qrc.status {
        TaskStatus::NeedsCreation => match modify_mode {
            true => {
                let crc = task.dispatch(&handle, &TaskRequest::create());
                match crc.status {
                    TaskStatus::IsCreated => { crc },
                    TaskStatus::Failed  => { crc },
                    _=> { panic!("module internal fsm state invalid (on create)") }
                }
            },
            false => handle.is_created(&query)
        },
        TaskStatus::NeedsRemoval => match modify_mode {
            true => {
                let rrc = task.dispatch(&handle, &TaskRequest::remove());
                match rrc.status {
                    TaskStatus::IsRemoved => { rrc },
                    TaskStatus::Failed  => { rrc },
                    _=> { panic!("module internal fsm state invalid (on remove)") }
                }
            }, 
            false => handle.is_removed(&query),
        },
        TaskStatus::NeedsModification => match modify_mode {
            true => {
                let mrc = task.dispatch(&handle, &TaskRequest::modify(Arc::clone(&qrc.changes)));
                match mrc.status {
                    TaskStatus::IsModified => { mrc },
                    TaskStatus::Failed  => { mrc },
                    _=> { panic!("module internal fsm state invalid (on modify)") }
                }
            },
            false => handle.is_modified(&query, Arc::clone(&qrc.changes))
        },
        TaskStatus::Failed => qrc,
        _ => { panic!("module internal fsm state invalid (on query)"); }
    };


    return result;

}

