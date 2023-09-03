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

use crate::registry::list::Task;
use crate::connection::connection::Connection;
use crate::handle::handle::TaskHandle;
use crate::playbooks::traversal::RunState;
use crate::tasks::request::TaskRequest;
use crate::inventory::hosts::Host;
use crate::tasks::response::{TaskStatus,TaskResponse};
//use crate::tasks::logic::PreLogicInput;
use std::sync::{Arc,RwLock,Mutex};
use std::collections::HashMap;
use rayon::prelude::*;
use crate::playbooks::traversal::{FsmMode,HandlerMode};


pub fn fsm_run_task(run_state: &Arc<RunState>, task: &Task, are_handlers: HandlerMode, fsm_mode: FsmMode) -> Result<(), String> {

    let hosts : HashMap<String, Arc<RwLock<Host>>> = run_state.context.read().unwrap().get_remaining_hosts();
    if hosts.len() == 0 { return Err(String::from("no hosts remaining")) }
    let mut host_objects : Vec<Arc<RwLock<Host>>> = Vec::new();
    for (_,v) in hosts { host_objects.push(Arc::clone(&v)); }

    let _total : i64 = host_objects.par_iter().map(|host| {

        // the parallel threaded part that runs on each host

        let connection_result = run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host);
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                run_state.visitor.read().unwrap().on_host_task_start(&run_state.context, &host);
                let task_response = run_task_on_host(&run_state,&connection,&host,task,are_handlers,fsm_mode);

                match task_response {
                    Ok(x) => {
                        run_state.visitor.read().unwrap().on_host_task_ok(&run_state.context, &x, &host);
                    }
                    Err(x) => {
                        match fsm_mode { 
                            FsmMode::FullRun => { run_state.context.write().unwrap().fail_host(&host); }
                            FsmMode::SyntaxOnly => { run_state.context.write().unwrap().syntax_fail_host(&host); }
                        }
                        run_state.visitor.read().unwrap().on_host_task_failed(&run_state.context, &x, &host);
                    },
                }
            },
            Err(x) => {
                // connection failures cannot happen in syntax-only check modes so we don't need anything here
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
    task: &Task,
    are_handlers: HandlerMode, 
    fsm_mode: FsmMode) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {

    // FIXME: break into smaller functions...

    let play_count = run_state.context.read().unwrap().play_count;
    let modify_mode = ! run_state.visitor.read().unwrap().is_check_mode();
    let handle = Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(connection), Arc::clone(host)));
    let validate = TaskRequest::validate();
    let evaluated = task.evaluate(&handle, &validate)?;

    let action = evaluated.action;
    let pre_logic = evaluated.with;
    let post_logic = evaluated.and;

    if fsm_mode == FsmMode::SyntaxOnly {
        if are_handlers == HandlerMode::Handlers  {
            if pre_logic.is_none() || pre_logic.as_ref().as_ref().unwrap().subscribe.is_none() {
                return Err(handle.response.is_failed(&Arc::clone(&validate), &String::from("with/subscribe missing in handler task definition")));
            }
        }
        return Ok(handle.response.is_matched(&Arc::clone(&validate)));
    }

    if pre_logic.is_some() {
        let logic = pre_logic.as_ref().as_ref().unwrap();
        let my_host = host.read().unwrap();
        if are_handlers == HandlerMode::Handlers  {
            if ! my_host.is_notified(play_count, &logic.subscribe.as_ref().unwrap().clone()) {
                return Ok(handle.response.is_skipped(&Arc::clone(&validate))); 
            } else {
            }
        }
        if ! logic.cond {
            return Ok(handle.response.is_skipped(&Arc::clone(&validate)));
        }
    }

    // this looks like overkill but there's a lot of extra checking to make sure modules
    // don't return the wrong states, even when returning an error, to prevent
    // unpredictability in the program

    // FIXME: break up into smaller functions
    let query = TaskRequest::query();
    let qrc = action.dispatch(&handle, &TaskRequest::query());

    let (_request, result) : (Arc<TaskRequest>, Result<Arc<TaskResponse>,Arc<TaskResponse>>) = match qrc {
        Ok(ref qrc_ok) => match qrc_ok.status {
            TaskStatus::IsMatched => {
                (Arc::clone(&query), Ok(handle.response.is_matched(&Arc::clone(&query))))
            },
            TaskStatus::NeedsCreation => match modify_mode {
                true => {
                    let req = TaskRequest::create();
                    let crc = action.dispatch(&handle, &req);
                    match crc {
                        Ok(ref crc_ok) => match crc_ok.status {
                            TaskStatus::IsCreated => (req, crc),
                            // these are all module coding errors, should they occur, and cannot happen in normal operation
                            _ => { panic!("(a) module internal fsm state invalid (on create): {:?}", crc); }
                        },
                        Err(ref crc_err) => match crc_err.status {
                            TaskStatus::Failed  => (req, crc),
                            _ => { panic!("(b) module internal fsm state invalid (on create), {:?}", crc); }
                        }
                    }
                },
                false => (Arc::clone(&query), Ok(handle.response.is_created(&Arc::clone(&query))))
            },
            TaskStatus::NeedsRemoval => match modify_mode {
                true => {
                    let req = TaskRequest::remove();
                    let rrc = action.dispatch(&handle, &req);
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
                false => (Arc::clone(&query), Ok(handle.response.is_removed(&Arc::clone(&query)))),
            },
            TaskStatus::NeedsModification => match modify_mode {
                true => {
                    let req = TaskRequest::modify(qrc_ok.changes.clone());
                    let mrc = action.dispatch(&handle, &req);
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
                false => (Arc::clone(&query), Ok(handle.response.is_modified(&Arc::clone(&query), qrc_ok.changes.clone())))
            },
            TaskStatus::NeedsExecution => match modify_mode {
                true => {
                    let req = TaskRequest::execute();
                    let erc = action.dispatch(&handle, &req);
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
                false => (Arc::clone(&query), Ok(handle.response.is_executed(&Arc::clone(&query))))
            },
            TaskStatus::NeedsPassive => match modify_mode {
                true => {
                    let req = TaskRequest::passive();
                    let prc = action.dispatch(&handle, &req);
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
                false => (Arc::clone(&query), Ok(handle.response.is_executed(&Arc::clone(&query))))
            },
            TaskStatus::Failed => { panic!("module returned failure inside an Ok(): {:?}", qrc); },
            _ => { panic!("module internal fsm state unknown (on query): {:?}", qrc); }
        },
        Err(x) => match x.status {
            TaskStatus::Failed => (query, Err(x)),
            _ => { panic!("module returned a non-failure code inside an Err: {:?}", x); }
        }
    };


    if result.is_ok() {
        if post_logic.is_some() {
            let logic = post_logic.as_ref().as_ref().unwrap();
            if are_handlers == HandlerMode::NormalTasks && result.is_ok() {
                let notify = logic.notify.as_ref().unwrap().clone();

                let status = &result.as_ref().unwrap().status;
                match status {
                    TaskStatus::IsCreated | TaskStatus::IsModified | TaskStatus::IsRemoved | TaskStatus::IsExecuted => {
                        run_state.visitor.read().unwrap().on_notify_handler(host, &notify.clone());
                        host.write().unwrap().notify(play_count, &notify.clone());
                    },
                    _ => { }
                }
            }
        }
    }

    // FIXME: apply post-logic ("and") to result here (in function)

    return result;

}
