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

use crate::module_base::common::*

/*

pub enum TaskRequestType {
    Validate,Query,Create,Remove,Modify,
}

pub struct TaskRequest {
    pub request_type: TaskRequestType,
    pub changes: Option<HashMap<String, String>>
}

pub enum TaskStatus {
    Validated,NeedsCreation,NeedsRemoval,NeedsModification,Done,Failed
}

pub struct TaskResponse {
    pub is: TaskStatus,pub changes: Option<HashMap<String, String>>
}

*/

// run a task on one or more hosts -- check modes (syntax/normal), or for 'real', on any connection type
pub fn run_task(
    context: &PlaybookContext,
    visitor: &dyn PlaybookVisitor, 
    connection_factory: &dyn ConnectionFactory, 
    task: &Task) -> Result<(), String> {

    let task_result = visitor.is_syntax_only();


    if syntax {
        let sc = run_task_on_hosts(context, visitor, connection, task);
        match sc.is {
            TaskStatus::Validated => { return Ok(()) } 
            _ => { Err(format!("resource defintion integrity check failed: {}", sc.msg )) }
        }
    }

    hosts = context.get_all_hosts();
    for host in hosts {
        let connection = connection_factory.get_connection(host);
        run_task_on_host(context, visitor, connection, task);
    }

    return Ok(());

    // FIXME: the results per host should be set on the context, and if any failures arise,
    // we know to take them *out* of the all_hosts_pool and record the failures


}

fn task_dispatch(task &Task, 
    context: &PlaybookContext, 
    visitor: &PlaybookVisitor, 
    connection: &dyn Connection, 
    request_type: TaskRequestType) {

    let task_handle = TaskHandle::new(context, visitor, connection, request_type)
    return task.dispatch(task_handle, TaskRequest { request_type: request_type, changes: None });

}

fn run_task_on_host(
    context: &PlaybookContext,
    visitor: &dyn PlaybookVisitor, 
    connection: &dyn Connection, 
    task: &Task) -> TaskResponse {

    let syntax     = visitor.is_syntax_only();
    let check_mode = visitor.is_check_mode();

    let vrc = task_dispatch(&task, context, visitor, connection, TaskRequestType::Validate);
    match vrc.is {
        TaskStatus::Validated => { vrc },
        TaskStatus::Failed => { vrc },
        _ => { panic!("module internal fsm state invalid (on verify)"); 
    }

    if syntax_mode || vrc.is == TaskStatus::Failed {
        return vrc;
    }

    qrc = task_dispatch(&task, context, visitor, connection, TaskRequestType::Query);
    let result = match vrc.is {
        TaskStatus::NeedsCreation => {
            match modify_mode {
                true => {
                    let crc = task_dispatch(&task, context, visitor, connection, TaskRequestType::Create);
                    match crc.is {
                        TaskStatus::Created => { crc },
                        TaskStatus::Failed  => { crc },
                        _=> { panic!("module internal fsm state invalid (on create)"); }
                    }
                }
                false => is_created();
            }
        }
        TaskStatus::NeedsRemoval => {
            match modify_mode {
                true => {
                    let rrc = task_dispatch(&task, context, visitor, connection, TaskRequestType::Remove);
                    match rrc.is {
                        TaskStatus::Removed => { rrc },
                        TaskStatus::Failed  => { rrc },
                        _=> { panic!("module internal fsm state invalid (on remove)"); }
                }, 
                false => is_removed(),
            }
        }
        TaskStatus::NeedsModification => {
            match modify_mode {
                true => {
                    let mrc = task_dispatch(&task, context, visitor, connection, TaskRequestType::Modify);
                    match mrc.is {
                        TaskStatus::Modified => { mrc },
                        TaskStatus::Failed  => { mrc },
                        _=> { panic!("module internal fsm state invalid (on modify)"); }
                    }
                }
                false => is_modified()
            }
        },
        _ => { panic!("module internal fsm state invalid (on query)");  
    }
    return result;

}

