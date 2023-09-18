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

// this module contains the guts of running tasks inside per-host threads
// while the actual core finite state machine is not terribly complicated
// various logical constructs in the language tend to cause lots of exceptions
//
// FIXME: this will be gradually refactored over time

pub fn fsm_run_task(run_state: &Arc<RunState>, play: &Play, task: &Task, are_handlers: HandlerMode) -> Result<(), String> {

    // if running in check mode various functions will short circuit early
    let check =  run_state.visitor.read().unwrap().is_check_mode();

    // the hosts to configure are not those specified in the batch but the subset of those that have not yet failed
    let hosts : HashMap<String, Arc<RwLock<Host>>> = run_state.context.read().unwrap().get_remaining_hosts();
    let mut host_objects : Vec<Arc<RwLock<Host>>> = Vec::new();
    for (_,v) in hosts { host_objects.push(Arc::clone(&v)); }

    // use rayon to process hosts in different threads
    let _total : i64 = host_objects.par_iter().map(|host| {

        // get the connection to each host, which should be left open until the play ends
        let connection_result = run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host);
        match connection_result {
            Ok(_)  => {
                let connection = connection_result.unwrap();
                run_state.visitor.read().unwrap().on_host_task_start(&run_state.context, &host);
                // the actual task is invoked here
                let task_response = run_task_on_host(&run_state,connection,&host,play,task,are_handlers);

                match task_response {
                    Ok(x) => {
                        match check {
                            // output slightly differs in check vs non-check modes
                            false => run_state.visitor.read().unwrap().on_host_task_ok(&run_state.context, &x, &host),
                            true => run_state.visitor.read().unwrap().on_host_task_check_ok(&run_state.context, &x, &host)
                        }
                    }
                    Err(x) => {
                        // hosts with task failures are removed from the pool
                        run_state.context.write().unwrap().fail_host(&host);
                        run_state.visitor.read().unwrap().on_host_task_failed(&run_state.context, &x, &host);
                    },
                }
            },
            Err(x) => {
                // hosts with connection failures are removed from the pool
                run_state.visitor.read().unwrap().debug_host(&host, &x);
                run_state.context.write().unwrap().fail_host(&host);
                run_state.visitor.read().unwrap().on_host_connect_failed(&run_state.context, &host);
            }
        }
        // rayon needs some math to add up, hence the 1. It seems to short-circuit without some work to do.
        return 1;

    }).sum();
    return Ok(());
}

fn get_actual_connection(run_state: &Arc<RunState>, host: &Arc<RwLock<Host>>, task: &Task, input_connection: Arc<Mutex<dyn Connection>>) -> Result<(Option<String>,Arc<Mutex<dyn Connection>>), String> {
    
    // usually the connection we already have is the one we will use, but this is not the case for using the delegate_to feature
    // this is a bit complex...

    return match task.get_with() {
        
        // if the task has a with section then the task might be delegated
        Some(task_with) => match task_with.delegate_to {

            // we have found the delegate_to keyword
            Some(pre_delegate) => {

                // we need to store the variable 'delegate_host' into the host's facts storage so it can be used in module parameters.
                let hn = host.read().unwrap().name.clone();
                let mut mapping = serde_yaml::Mapping::new();
                mapping.insert(serde_yaml::Value::String(String::from("delegate_host")), serde_yaml::Value::String(hn.clone()));
                host.write().unwrap().update_facts2(mapping);
                
                // the delegate_to parameter could be a variable
                let delegate = run_state.context.read().unwrap().render_template(&pre_delegate, host, BlendTarget::NotTemplateModule, TemplateMode::Strict)?;

                if delegate.eq(&hn.clone()) {
                    // delegating to the same host will deadlock since the connection is wrapped in a mutex, 
                    // so just return the original connection if that is requested
                    return Ok((None, input_connection))
                }
                else if delegate.eq(&String::from("localhost")) {
                    // localhost delegation has some security implications (see docs) so require a CLI flag for access
                    if run_state.allow_localhost_delegation {
                        return Ok((Some(delegate.clone()), run_state.connection_factory.read().unwrap().get_local_connection(&run_state.context)?))
                    } else {
                        return Err(format!("localhost delegation has potential security implementations, pass --allow-localhost-delegation to sign off"));
                    }
                }
                else {
                    // with some pre-checks out of the way, allow delegation to the host if it's in inventory
                    let has_host = run_state.inventory.read().unwrap().has_host(&delegate);
                    if ! has_host {
                        return Err(format!("cannot delegate to a host not found in inventory: {}", delegate));
                    }
                    let host = run_state.inventory.read().unwrap().get_host(&delegate);
                    return Ok((Some(delegate.clone()), run_state.connection_factory.read().unwrap().get_connection(&run_state.context, &host)?));
                } 
            },
            // there was no delegate keyword, use the original connection
            None => Ok((None, input_connection))
        },
        // there was no 'with' block, use teh original connection
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

    // to run a task we must first validate the object, which renders the YAML inputs into versions where the program
    // has applied more pre-processing
    let validate = TaskRequest::validate();

    // consider the use of the delegate_to keyword, if provided
    let gac_result = get_actual_connection(run_state, host, task, Arc::clone(&input_connection));

    let (delegated, connection, handle) = match gac_result {
        // construct the TaskHandle if the original connection is to be used
        Ok((None, ref conn)) => (None, conn, Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(conn), Arc::clone(host)))),
        // construct the TaskHandle if a delegate connection is to be used
        Ok((Some(delegate), ref conn)) => (Some(delegate.clone()), conn, Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(conn), Arc::clone(host)))),
        // something went wrong when processing delegates, create a throw-away handle just so we can use the response functions
        Err(msg) => {
            let tmp_handle = Arc::new(TaskHandle::new(Arc::clone(run_state), Arc::clone(&input_connection), Arc::clone(host)));
            return Err(tmp_handle.response.is_failed(&validate, &msg));
        }
    };

    // if we are delegating, tell the user
    if delegated.is_some() {
        run_state.visitor.read().unwrap().on_host_delegate(host, &delegated.unwrap());
    }

    // process the YAML inputs of the task and turn them into something we can  use
    // initially we run this in 'template off' mode which returns basically junk
    // but allows us to get the 'items' data off the collection. 
    let evaluated = task.evaluate(&handle, &validate, TemplateMode::Off)?;

    // see if we are iterating over a list of items or not
    let items_input = match evaluated.with.is_some() {
        true => &evaluated.with.as_ref().as_ref().unwrap().items,
        false => &None
    };

    // mapping to store the 'item' variable when using 'with_items'
    let mut mapping = serde_yaml::Mapping::new();

    // storing the last result of the items loop so we always have something to return
    // if a failure occurs it will be returned immediately
    let mut last : Option<Result<Arc<TaskResponse>,Arc<TaskResponse>>> = None;

    // even if we are not iterating over a list of items, make a list of one item to simplify the logic
    let evaluated_items = template_items(&handle, &validate, TemplateMode::Strict, &items_input)?;

    // walking over each item or just the single task if 'with_items' was not used
    for item in evaluated_items.iter() {
            
        // store the 'items' variable for use in module parameters
        mapping.insert(serde_yaml::Value::String(String::from("item")), item.clone());
        host.write().unwrap().update_facts2(mapping.clone());

        // re-evaluate the task, allowing the 'items' to be plugged in.
        let evaluated = task.evaluate(&handle, &validate, TemplateMode::Strict)?;

        // see if there is any retry or delay logic in the task
        let mut retries = match evaluated.and.as_ref().is_some() {
            false => 0, true => evaluated.and.as_ref().as_ref().unwrap().retry
        };
            let delay = match evaluated.and.as_ref().is_some() {
            false => 1, true => evaluated.and.as_ref().as_ref().unwrap().delay
        };
    
        // run the task as many times as defined by retry logic
        loop {
            
            // here we finally call the actual task, everything around this is just support
            // for delegation, loops, and retries!
            match run_task_on_host_inner(run_state, &connection, host, play, task, are_handlers, &handle, &validate, &evaluated) {
                Err(e) => match retries {
                    // retries are used up
                    0 => { return Err(e); },
                    // we have retries left
                    _ => { 
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

    // looping over a list of no items should be impossible unless someone passed in a variable that was
    // an empty list
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

    // access any pre and post-task modifier logic
    let action = &evaluated.action;
    let pre_logic = &evaluated.with;
    let post_logic = &evaluated.and;

    // get the sudo settings from the play if available, if not see if they were set from the CLI
    let mut sudo : Option<String> = match play.sudo.is_some() {
        true => play.sudo.clone(),
        // minor FIXME: parameters like this are usually set on the run_state
        false => run_state.context.read().unwrap().sudo.clone() 
    };
    // see if the sudo template is configured, if not use the most basic default
    let sudo_template = match &play.sudo_template {
        None => String::from("/usr/bin/sudo -u '{{jet_sudo_user}}' {{jet_command}}"),
        Some(x) => x.clone()
    };
    
    // is 'with' provided?
    if pre_logic.is_some() {
        let logic = pre_logic.as_ref().as_ref().unwrap();
        let my_host = host.read().unwrap();
        if are_handlers == HandlerMode::Handlers  {
            // if we are running handlers at the moment, skip any un-notified handlers
            if ! my_host.is_notified(play_count, &logic.subscribe.as_ref().unwrap().clone()) {
                return Ok(handle.response.is_skipped(&Arc::clone(&validate))); 
            }
        }
        // if a condition was provided and it was false, skip the task
        // lack of a condition provides a 'true' condition, hence no use of option processing here
        if ! logic.condition {
            return Ok(handle.response.is_skipped(&Arc::clone(&validate)));
        }
        // if sudo was requested on the specific task override any sudo computations above
        if logic.sudo.is_some() {
            sudo = Some(logic.sudo.as_ref().unwrap().clone());
        }
    }

    let sudo_details = SudoDetails {
        user     : sudo.clone(),
        template : sudo_template.clone()
    };

    // we're about to get to the task finite state machine guts.
    // this looks like overkill but there's a lot of extra checking to make sure modules
    // don't return the wrong states, even when returning an error, to prevent
    // unpredictability in the program

    let query = TaskRequest::query(&sudo_details);

    // invoke the resource and see what actions it thinks need to be performed

    let qrc = action.dispatch(&handle, &query);

    // in check mode we short-circuit evaluation early, except for passive modules
    // like 'facts'

    if run_state.visitor.read().unwrap().is_check_mode() {
        match qrc {
            Ok(ref qrc_ok) => match qrc_ok.status {
                TaskStatus::NeedsPassive => { /* allow modules like !facts or set to execute */ },
                _ => { return qrc; }
            },
            _ => {}
        }
    }

    // with the query completed, what action to perform next depends on the query results

    let (_request, prelim_result) : (Arc<TaskRequest>, Result<Arc<TaskResponse>,Arc<TaskResponse>>) = match qrc {
        Ok(ref qrc_ok) => match qrc_ok.status {

            // matched indicates we don't need to do anything
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

            // these panic states should never really happen unless there is a module coding error
            // it is unacceptable for a module to deliberately panic, they should
            // always return a TaskResponse.

            TaskStatus::Failed => { panic!("module returned failure inside an Ok(): {:?}", qrc); },           
            _ => { panic!("module internal fsm state unknown (on query): {:?}", qrc); }

        },
        Err(x) => match x.status {
            TaskStatus::Failed => (query, Err(x)),
            _ => { panic!("module returned a non-failure code inside an Err: {:?}", x); }
        }
    };

    // now that we've got a result, whether we use that result depends
    // on whether ignore_errors was set.

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

    // if and/notify is present, notify handlers when changed actions are seen

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

    // ok, we're done, whew

    return result;
}
