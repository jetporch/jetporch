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
use crate::inventory::hosts::Host;
use crate::playbooks::traversal::HandlerMode;
use crate::playbooks::language::Play;
use crate::tasks::request::SudoDetails;
use crate::tasks::*;
use crate::handle::template::BlendTarget;
use crate::playbooks::templar::TemplateMode;
use crate::tasks::logic::template_items;
use std::sync::{Arc,RwLock,Mutex};
use std::collections::HashMap;
use rayon::prelude::*;
use std::{thread, time};


pub fn fsm_run_task(run_state: &Arc<RunState>, play: &Play, task: &Task, are_handlers: HandlerMode) -> Result<(), String> {

    let check =  run_state.visitor.read().unwrap().is_check_mode();
    let hosts : HashMap<String, Arc<RwLock<Host>>> = run_state.context.read().unwrap().get_remaining_hosts();
    //if hosts.len() == 0 { return Err(String::from("no hosts remaining")) }
    let mut host_objects : Vec<Arc<RwLock<Host>>> = Vec::new();
    for (_,v) in hosts { host_objects.push(Arc::clone(&v)); }

    let _total : i64 = host_objects.par_iter().map(|host| {

        // the parallel threaded part that runs on each host

        let connection_result = run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host);
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                run_state.visitor.read().unwrap().on_host_task_start(&run_state.context, &host);
                let task_response = run_task_on_host(&run_state,connection,&host,play,task,are_handlers);

                match task_response {
                    Ok(x) => {
                        match check {
                            false => run_state.visitor.read().unwrap().on_host_task_ok(&run_state.context, &x, &host),
                            true => run_state.visitor.read().unwrap().on_host_task_check_ok(&run_state.context, &x, &host)
                        }
                    }
                    Err(x) => {
                        run_state.context.write().unwrap().fail_host(&host);
                        run_state.visitor.read().unwrap().on_host_task_failed(&run_state.context, &x, &host);
                    },
                }
            },
            Err(x) => {
                run_state.visitor.read().unwrap().debug_host(&host, &x);
                run_state.context.write().unwrap().fail_host(&host);
                run_state.visitor.read().unwrap().on_host_connect_failed(&run_state.context, &host);
            }
        }
        return 1;

    }).sum();
    return Ok(());
}

fn get_actual_connection(run_state: &Arc<RunState>, host: &Arc<RwLock<Host>>, task: &Task, input_connection: Arc<Mutex<dyn Connection>>) -> Result<(Option<String>,Arc<Mutex<dyn Connection>>), String> {
    return match task.get_with() {
        Some(task_with) => match task_with.delegate_to {
            Some(pre_delegate) => {
                let hn = host.read().unwrap().name.clone();
                let mut mapping = serde_yaml::Mapping::new();
                mapping.insert(serde_yaml::Value::String(String::from("delegate_host")), serde_yaml::Value::String(hn.clone()));
                host.write().unwrap().update_facts2(mapping);
                
                let delegate = run_state.context.read().unwrap().render_template(&pre_delegate, host, BlendTarget::NotTemplateModule, TemplateMode::Strict)?;

                if delegate.eq(&hn.clone()) {
                    return Ok((None, input_connection))
                }
                else if delegate.eq(&String::from("localhost")) {
                    if run_state.allow_localhost_delegation {
                        return Ok((Some(delegate.clone()), run_state.connection_factory.read().unwrap().get_local_connection(&run_state.context)?))
                    } else {
                        return Err(format!("localhost delegation has potential security implementations, pass --allow-localhost-delegation to sign off"));
                    }
                }
                else {
                    let has_host = run_state.inventory.read().unwrap().has_host(&delegate);
                    if ! has_host {
                        return Err(format!("cannot delegate to a host not found in inventory: {}", delegate));
                    }
                    let host = run_state.inventory.read().unwrap().get_host(&delegate);
                    return Ok((Some(delegate.clone()), run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host)?));
                } 
            },
            None => Ok((None, input_connection))
        },
        None => Ok((None, input_connection))
    };
}

fn run_task_on_host(
    run_state: &Arc<RunState>,
    input_connection: Arc<Mutex<dyn Connection>>,
    host: &Arc<RwLock<Host>>,
    play: &Play, 
    task: &Task,
    are_handlers: HandlerMode) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {

    let validate = TaskRequest::validate();

    let gac_result = get_actual_connection(run_state, host, task, Arc::clone(&input_connection));

    let (delegated, connection, handle) = match gac_result {
        Ok((None, ref conn)) => (None, conn, Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(conn), Arc::clone(host)))),
        Ok((Some(delegate), ref conn)) => (Some(delegate.clone()), conn, Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(conn), Arc::clone(host)))),
        Err(msg) => {
            let tmp_handle = Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(&input_connection), Arc::clone(host)));
            return Err(tmp_handle.response.is_failed(&validate, &msg));
        }
    };

    if delegated.is_some() {
        run_state.visitor.read().unwrap().on_host_delegate(host, &delegated.unwrap());
    }
 
    let evaluated = task.evaluate(&handle, &validate, TemplateMode::Off)?;

    let items_input = match evaluated.with.is_some() {
        true => &evaluated.with.as_ref().as_ref().unwrap().items,
        false => &None
    };

    let mut mapping = serde_yaml::Mapping::new();
    let mut last : Option<Result<Arc<TaskResponse>,Arc<TaskResponse>>> = None;

    let evaluated_items = template_items(&handle, &validate, TemplateMode::Strict, &items_input)?;

    for item in evaluated_items.iter() {
            
        mapping.insert(serde_yaml::Value::String(String::from("item")), item.clone());
        host.write().unwrap().update_facts2(mapping.clone());

        let evaluated = task.evaluate(&handle, &validate, TemplateMode::Strict)?;

        let mut retries = match evaluated.and.as_ref().is_some() {
            false => 0, true => evaluated.and.as_ref().as_ref().unwrap().retry
        };
    
        let delay = match evaluated.and.as_ref().is_some() {
            false => 1, true => evaluated.and.as_ref().as_ref().unwrap().delay
        };
    
        loop {
            match run_task_on_host_inner(run_state, &connection, host, play, task, are_handlers, &handle, &validate, &evaluated) {
                Err(e) => match retries {
                    0 => { return Err(e); },
                    _ => { 
                        //println!("***** RETRIES: {}", retries);
                        retries = retries - 1;
                        run_state.visitor.read().unwrap().on_host_task_retry(&run_state.context, host, retries, delay);
                        if delay > 0 {
                            let duration = time::Duration::from_secs(delay);
                            thread::sleep(duration);
                        }
                    }
                },
                Ok(x) => { last = Some(Ok(x)); break }
            }
        }
    
    }

    if last.is_some() {
        return last.unwrap();
    }
    else {
        return Err(handle.response.is_failed(&validate, &String::from("with/items contained no entries")));    
    }

}

// the "on this host" method body from _task
fn run_task_on_host_inner(
    run_state: &Arc<RunState>,
    _connection: &Arc<Mutex<dyn Connection>>,
    host: &Arc<RwLock<Host>>,
    play: &Play, 
    _task: &Task,
    are_handlers: HandlerMode, 
    handle: &Arc<TaskHandle>,
    validate: &Arc<TaskRequest>,
    evaluated: &EvaluatedTask) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {

    let play_count = run_state.context.read().unwrap().play_count;
    let modify_mode = ! run_state.visitor.read().unwrap().is_check_mode();

    let action = &evaluated.action;
    let pre_logic = &evaluated.with;
    let post_logic = &evaluated.and;

    let mut sudo : Option<String> = match play.sudo.is_some() {
        true => play.sudo.clone(),
        false => run_state.context.read().unwrap().sudo.clone() 
    };
    let sudo_template = match &play.sudo_template {
        None => String::from("/usr/bin/sudo -u '{{jet_sudo_user}}' {{jet_command}}"),
        Some(x) => x.clone()
    };
    
    if pre_logic.is_some() {
        let logic = pre_logic.as_ref().as_ref().unwrap();
        let my_host = host.read().unwrap();
        if are_handlers == HandlerMode::Handlers  {
            if ! my_host.is_notified(play_count, &logic.subscribe.as_ref().unwrap().clone()) {
                return Ok(handle.response.is_skipped(&Arc::clone(&validate))); 
            } else {
            }
        }
        if ! logic.condition {
            return Ok(handle.response.is_skipped(&Arc::clone(&validate)));
        }
        if logic.sudo.is_some() {
            sudo = Some(logic.sudo.as_ref().unwrap().clone());
        }
    }

    let sudo_details = SudoDetails {
        user     : sudo.clone(),
        template : sudo_template.clone()
    };

    // this looks like overkill but there's a lot of extra checking to make sure modules
    // don't return the wrong states, even when returning an error, to prevent
    // unpredictability in the program

    let query = TaskRequest::query(&sudo_details);
    let qrc = action.dispatch(&handle, &query);

    if run_state.visitor.read().unwrap().is_check_mode() {
        match qrc {
            Ok(ref qrc_ok) => match qrc_ok.status {
                TaskStatus::NeedsPassive => { /* allow modules like !facts or set to execute */ },
                _ => { return qrc; }
            },
            _ => {}
        }
    }

    let (_request, prelim_result) : (Arc<TaskRequest>, Result<Arc<TaskResponse>,Arc<TaskResponse>>) = match qrc {
        Ok(ref qrc_ok) => match qrc_ok.status {
            TaskStatus::IsMatched => {
                (Arc::clone(&query), Ok(handle.response.is_matched(&Arc::clone(&query))))
            },
            TaskStatus::NeedsCreation => match modify_mode {
                true => {
                    let req = TaskRequest::create(&sudo_details);
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
                    let req = TaskRequest::remove(&sudo_details);
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
                    let req = TaskRequest::modify(&sudo_details, qrc_ok.changes.clone());
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
                    let req = TaskRequest::execute(&sudo_details);
                    let erc = action.dispatch(&handle, &req);
                    match erc {
                        Ok(ref erc_ok) => match erc_ok.status {
                            TaskStatus::IsExecuted => (req, erc),
                            TaskStatus::IsPassive => (req, erc),
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
            TaskStatus::NeedsPassive => {
                let req = TaskRequest::passive(&sudo_details);
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
            TaskStatus::Failed => { panic!("module returned failure inside an Ok(): {:?}", qrc); },
            _ => { panic!("module internal fsm state unknown (on query): {:?}", qrc); }
        },
        Err(x) => match x.status {
            TaskStatus::Failed => (query, Err(x)),
            _ => { panic!("module returned a non-failure code inside an Err: {:?}", x); }
        }
    };

    let result = match prelim_result {
        Ok(x) => Ok(x), 
        Err(y) =>  {
            if post_logic.is_some() {
                let logic = post_logic.as_ref().as_ref().unwrap();
                match logic.ignore_errors {
                    true => Ok(y),
                    false => Err(y)
                }
            }
            else {
                Err(y)
            }
        }
    };

    if result.is_ok() {
        if post_logic.is_some() {
            let logic = post_logic.as_ref().as_ref().unwrap();
            if are_handlers == HandlerMode::NormalTasks && result.is_ok() && logic.notify.is_some() {
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

    return result;
}
