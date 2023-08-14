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

use crate::module_base::common::{TaskStatus,TaskResponse};
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;
use crate::connection::factory::ConnectionFactory;
use crate::module_base::list::Task;
use crate::connection::no::NoConnection;
use crate::connection::connection::Connection;
use crate::runner::task_handle::TaskHandle;
use crate::module_base::common::TaskRequest;
use std::sync::Arc;
use std::sync::Mutex;

// run a task on one or more hosts -- check modes (syntax/normal), or for 'real', on any connection type

pub fn fsm_run_task(
    context: Arc<Mutex<PlaybookContext>>,
    visitor: Arc<Mutex<dyn PlaybookVisitor>>, 
    connection_factory: Arc<Mutex<dyn ConnectionFactory>>, 
    task: Arc<Task>) -> Result<(), String> {

    let no_connection = Arc::new(Mutex::new(NoConnection::new()));
    let syntax_check_result = run_task_on_host(context, visitor, no_connection, task);
    match syntax_check_result.is {
        TaskStatus::IsValidated => { 
            if visitor.lock().unwrap().is_syntax_only() { return Ok(()); }
        }, 
        TaskStatus::Failed => { 
            return Err(format!("parameters conflict: {}", syntax_check_result.msg.unwrap()));
        },
        _ => { panic!("module returned invalid response to syntax check") }
    }

    let hosts = context.lock().unwrap().get_all_hosts();
    for host in hosts {
        let connection_result = connection_factory.lock().unwrap().get_connection(context, host);
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                let task_response = run_task_on_host(context, visitor, connection, task);
                if task_response.is_failed() {
                    visitor.lock().unwrap().on_host_task_failed(context, Arc::new(task_response), host);
                }
            },
            Err(_) => { 
                visitor.lock().unwrap().on_host_connect_failed(context, host);
            }
        }
    }
    return Ok(());
}

// a wrapper around task.dispatch that provides the module
// with a TaskHandle object rather than direct access to all methods
// in playbook visitor and context, so that modules mostly do
// the right thing and stay standardized

/*
fn task_dispatch(task: &Task, 
    context: &PlaybookContext, 
    visitor: &PlaybookVisitor, 
    connection: &dyn Connection, 
    request_type: TaskRequestType) {

    let task_handle = TaskHandle::new(context, visitor, connection, request_type);
    return task.dispatch(task_handle, TaskRequest { request_type: request_type, changes: None });

}
*/

// the "on this host" method body from fsm_run_task

fn run_task_on_host(
    context: Arc<Mutex<PlaybookContext>>,
    visitor: Arc<Mutex<dyn PlaybookVisitor>>, 
    connection: Arc<Mutex<dyn Connection>>, 
    task: Arc<Task>) -> TaskResponse {

    let syntax      = visitor.lock().unwrap().is_syntax_only();
    let modify_mode = ! visitor.lock().unwrap().is_check_mode();

    let handle = Arc::new(TaskHandle::new(context, visitor, connection));

    let vrc = task.dispatch(handle, TaskRequest::validate());
    match vrc.is {
        TaskStatus::IsValidated => { 
            if syntax {
                return vrc;
            }
        },
        TaskStatus::Failed => { return vrc; },
        _ => { panic!("module internal fsm state invalid (on verify)") }
    }


    let query = TaskRequest::query();
    let qrc = task.dispatch(handle, TaskRequest::query());
    let result = match qrc.is {
        TaskStatus::NeedsCreation => match modify_mode {
            true => {
                let crc = task.dispatch(handle, TaskRequest::create());
                match crc.is {
                    TaskStatus::IsCreated => { crc },
                    TaskStatus::Failed  => { crc },
                    _=> { panic!("module internal fsm state invalid (on create)") }
                }
            },
            false => handle.is_created(query)
        },
        TaskStatus::NeedsRemoval => match modify_mode {
            true => {
                let rrc = task.dispatch(handle, TaskRequest::remove());
                match rrc.is {
                    TaskStatus::IsRemoved => { rrc },
                    TaskStatus::Failed  => { rrc },
                    _=> { panic!("module internal fsm state invalid (on remove)") }
                }
            }, 
            false => handle.is_removed(query),
        },
        TaskStatus::NeedsModification => match modify_mode {
            true => {
                let mrc = task.dispatch(handle, TaskRequest::modify(Arc::clone(&qrc.changes)));
                match mrc.is {
                    TaskStatus::IsModified => { mrc },
                    TaskStatus::Failed  => { mrc },
                    _=> { panic!("module internal fsm state invalid (on modify)") }
                }
            },
            false => handle.is_modified(query, Arc::clone(&qrc.changes))
        },
        TaskStatus::Failed => qrc,
        _ => { panic!("module internal fsm state invalid (on query)"); }
    };
    return result;

}

