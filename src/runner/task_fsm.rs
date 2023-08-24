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

use crate::connection::factory::ConnectionFactory;
use crate::connection::no::NoFactory;
use crate::registry::list::Task;
use crate::connection::connection::Connection;
use crate::tasks::handle::TaskHandle;
use crate::playbooks::traversal::RunState;
use crate::tasks::request::TaskRequest;
use crate::inventory::hosts::Host;
use crate::tasks::response::{TaskStatus,TaskResponse};
use std::sync::{Arc,RwLock,Mutex};
use std::collections::HashMap;
use rayon::prelude::*;

pub fn fsm_run_task(run_state: &Arc<RunState>, task: &Task, _are_handlers: bool) -> Result<(), String> {

    // syntax check first, always
    /*
    let tmp_localhost = Arc::new(RwLock::new(Host::new(&String::from("localhost"))));
    let no_connection = NoFactory::new().get_connection(&run_state.context, &tmp_localhost).unwrap();
    let syntax_check_result = run_task_on_host(run_state,&no_connection,&tmp_localhost,task, true);
    match syntax_check_result {
        Ok(scr_ok) => { //match scr_ok.status {
            //TaskStatus::IsValidated => { 
            //    if run_state.visitor.read().unwrap().is_syntax_only() { return Ok(()); }
            //}, 
            //_ => { panic!("module returned invalid response to syntax check (1): {:?}", scr_ok.as_ref()) }
        },
        Err(scr_err) => match scr_err.status {

            TaskStatus::Failed => { 
                println!("XDEBUG: Position 0");

                return Err(format!("parameters conflict: {}", scr_err.msg.as_ref().unwrap()));
            },
            _ => { panic!("module returned invalid response to syntax check (2): {:?}", scr_err.as_ref()) },
        }
    };
    let syntax = run_state.visitor.read().unwrap().is_syntax_only();
    if syntax {
        return Ok(())
    }
    */

    let hosts : HashMap<String, Arc<RwLock<Host>>> = run_state.context.read().unwrap().get_remaining_hosts();
    if hosts.len() == 0 { return Err(String::from("no hosts remaining")) }
    let mut host_objects : Vec<Arc<RwLock<Host>>> = Vec::new();
    for (_,v) in hosts { host_objects.push(Arc::clone(&v)); }

    let total : i64 = host_objects.par_iter().map(|host| {
        let connection_result = run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host);
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                run_state.visitor.read().unwrap().on_host_task_start(&run_state.context, &host);
                let task_response = run_task_on_host(&run_state,&connection,&host,task);
                
                match task_response {
                    Ok(x) => {
                        run_state.visitor.read().unwrap().on_host_task_ok(&run_state.context, &x, &host);
                    }
                    Err(x) => {
                        println!("XDEBUG: Position 2");
                        run_state.context.write().unwrap().fail_host(&host);
                        run_state.visitor.read().unwrap().on_host_task_failed(&run_state.context, &x, &host);
                    },
                }
            },
            Err(x) => {
                println!("XDEBUG: Position 3");

                run_state.visitor.read().unwrap().debug_host(&host, &x);
                run_state.context.write().unwrap().fail_host(&host);
                run_state.visitor.read().unwrap().on_host_connect_failed(&run_state.context, &host);
            }
        }
        return 1;
    }).sum();
    return Ok(());
}


// the "on this host" method body from fsm_run_task
fn run_task_on_host(
    run_state: &Arc<RunState>, 
    connection: &Arc<Mutex<dyn Connection>>,
    host: &Arc<RwLock<Host>>, 
    task: &Task) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {

    // FIXME: break into smaller functions...

    let modify_mode = ! run_state.visitor.read().unwrap().is_check_mode();
    let handle = Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(connection), Arc::clone(host)));
    let validate = TaskRequest::validate();
    let vrc = task.dispatch(&handle, &validate);
    match vrc {
        Ok(ref x) => match x.status {

            // FIXME: TODO:: isValidated means that vrc.logic contains a CommonLogic reference which we
            // can/must use to modify operations below, including possibly skipping them.

            TaskStatus::IsValidated => { 
                                
                if x.with.is_some() {
                    // handle 'cond' statement to decide if the task should be skipped
                    // FIXME: move to function
                    let logic = Arc::clone(&x.with);
                    // LOL, Rust
                    let cond = &logic.as_ref().as_ref().unwrap().cond;
                    if cond.is_some() {
                        let cond_value = &cond.as_ref().unwrap();
                        let test = handle.test_cond(&validate, &cond_value);
                        match test {
                            Ok(testp) => { 
                                if testp == false {
                                    return Ok(handle.is_skipped(&Arc::clone(&validate)));
                                }
                            },
                            Err(template_err) => {
                                return Err(template_err);
                            }
                        };
                    }
                    // ALSO need to process sudo here and other 'with' statements.
                }
        
            },
            TaskStatus::Failed => { panic!("module implementation returned a failed inside an Ok result") },
            _ => { panic!("module internal fsm state invalid (on verify)") }
        },
        Err(ref _x) => { return vrc }
    }

    let query = TaskRequest::query();
    let qrc = task.dispatch(&handle, &TaskRequest::query());
    let (request, result) : (Arc<TaskRequest>, Result<Arc<TaskResponse>,Arc<TaskResponse>>) = match qrc {
        Ok(ref qrc_ok) => match qrc_ok.status {
            TaskStatus::IsMatched => {
                (Arc::clone(&query), Ok(handle.is_matched(&Arc::clone(&query))))
            },
            TaskStatus::NeedsCreation => match modify_mode {
                true => {
                    let req = TaskRequest::create();
                    let crc = task.dispatch(&handle, &req);
                    match crc {
                        Ok(ref crc_ok) => match crc_ok.status {
                            TaskStatus::IsCreated => (req, crc),
                            _ => { panic!("module internal fsm state invalid (on create): {:?}", crc); }
                        },
                        Err(ref crc_err) => match crc_err.status {
                            TaskStatus::Failed  => (req, crc),
                            _ => { panic!("module internal fsm state invalid (on create), {:?}", crc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.is_created(&Arc::clone(&query))))
            },
            TaskStatus::NeedsRemoval => match modify_mode {
                true => {
                    let req = TaskRequest::remove();
                    let rrc = task.dispatch(&handle, &req);
                    match rrc {
                        Ok(ref rrc_ok) => match rrc_ok.status {
                            TaskStatus::IsRemoved => (req, rrc),
                            _ => { panic!("module internal fsm state invalid (on remove): {:?}", rrc); }
                        },
                        Err(ref rrc_err) => match rrc_err.status {
                            TaskStatus::Failed  => (req, rrc),
                            _ => { panic!("module internal fsm state invalid (on remove): {:?}", rrc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.is_removed(&Arc::clone(&query)))),
            },
            TaskStatus::NeedsModification => match modify_mode {
                true => {
                    let req = TaskRequest::modify(Arc::clone(&qrc_ok.changes));
                    let mrc = task.dispatch(&handle, &req);
                    match mrc {
                        Ok(ref mrc_ok) => match mrc_ok.status {
                            TaskStatus::IsModified => (req, mrc),
                            _ => { panic!("module internal fsm state invalid (on modify): {:?}", mrc); }
                        }
                        Err(ref mrc_err)  => match mrc_err.status {
                            TaskStatus::Failed  => (req, mrc),
                            _ => { panic!("module internal fsm state invalid (on modify): {:?}", mrc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.is_modified(&Arc::clone(&query), Arc::clone(&qrc_ok.changes))))
            },
            TaskStatus::NeedsExecution => match modify_mode {
                true => {
                    let req = TaskRequest::execute();
                    let erc = task.dispatch(&handle, &req);
                    match erc {
                        Ok(ref erc_ok) => match erc_ok.status {
                            TaskStatus::IsExecuted => (req, erc),
                            _ => { panic!("module internal fsm state invalid (on execute): {:?}", erc); }
                        }
                        Err(ref erc_err)  => match erc_err.status {
                            TaskStatus::Failed  => (req, erc),
                            _ => { panic!("module internal fsm state invalid (on execute): {:?}", erc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.is_executed(&Arc::clone(&query))))
            },
            TaskStatus::NeedsPassive => match modify_mode {
                true => {
                    let req = TaskRequest::passive();
                    let prc = task.dispatch(&handle, &req);
                    match prc {
                        Ok(ref prc_ok) => match prc_ok.status {
                            TaskStatus::IsPassive => (req, prc),
                            _ => { panic!("module internal fsm state invalid (on passive): {:?}", prc); }
                        }
                        Err(ref prc_err)  => match prc_err.status {
                            TaskStatus::Failed  => (req, prc),
                            _ => { panic!("module internal fsm state invalid (on passive): {:?}", prc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.is_executed(&Arc::clone(&query))))
            },
            TaskStatus::Failed => { panic!("module returned failure inside an Ok(): {:?}", qrc); },
            _ => { panic!("module internal fsm state unknown (on query): {:?}", qrc); }
        },
        Err(x) => match x.status {
            TaskStatus::Failed => (query, Err(x)),
            _ => { panic!("module returned a non-failure code inside an Err: {:?}", x); }
        }
    };

    // FIXME: apply post-logic ("and") to result here (in function)

    return result;

}

