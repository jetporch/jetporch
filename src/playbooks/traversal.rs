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

use crate::playbooks::language::Play;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::language::{Role,RoleInvocation};
use crate::connection::factory::ConnectionFactory;
use crate::registry::list::Task;
use crate::playbooks::task_fsm::fsm_run_task;
use crate::inventory::inventory::Inventory;
use crate::inventory::hosts::Host;
use crate::util::io::{jet_file_open,directory_as_string};
use crate::util::yaml::{blend_variables,show_yaml_error_in_context};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc,RwLock};
use std::path::Path;
use std::env;

#[derive(PartialEq,Copy,Debug,Clone)]
pub enum HandlerMode {
    NormalTasks,
    Handlers
}

pub struct RunState {
    pub inventory: Arc<RwLock<Inventory>>,
    pub playbook_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub role_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub limit_hosts: Vec<String>,
    pub limit_groups: Vec<String>,
    pub context: Arc<RwLock<PlaybookContext>>,
    pub visitor: Arc<RwLock<dyn PlaybookVisitor>>,
    pub connection_factory: Arc<RwLock<dyn ConnectionFactory>>,
    pub tags: Option<Vec<String>>
}

pub fn playbook_traversal(run_state: &Arc<RunState>) -> Result<(), String> {
        
    for playbook_path in run_state.playbook_paths.read().unwrap().iter() {

        { 
            let mut ctx = run_state.context.write().unwrap(); 
            ctx.set_playbook_path(playbook_path); 
        }

        run_state.visitor.read().unwrap().on_playbook_start(&run_state.context);

        let playbook_file = jet_file_open(&playbook_path)?;
        let parsed: Result<Vec<Play>, serde_yaml::Error> = serde_yaml::from_reader(playbook_file);
        if parsed.is_err() {
            show_yaml_error_in_context(&parsed.unwrap_err(), &playbook_path);
            return Err(format!("edit the file and try again?"));
        }   

        let p1 = env::current_dir().expect("could not get current directory");
        let previous = p1.as_path();
        

        let pbdirname = directory_as_string(playbook_path);
        let pbdir = Path::new(&pbdirname);
        if pbdirname.eq(&String::from("")) {
        } else {
            env::set_current_dir(&pbdir).expect("could not chdir into playbook directory");
        }

        let plays: Vec<Play> = parsed.unwrap();
        for play in plays.iter() {
            match handle_play(&run_state, play) {
                Ok(_) => {},
                Err(s) => { return Err(s); }
            }
            run_state.context.read().unwrap().connection_cache.write().unwrap().clear();
        }
        run_state.context.read().unwrap().connection_cache.write().unwrap().clear();

        env::set_current_dir(&previous).expect("could not restore previous directory");


    }
    run_state.context.read().unwrap().connection_cache.write().unwrap().clear();
    run_state.visitor.read().unwrap().on_exit(&run_state.context);
    return Ok(())
}

fn handle_play(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {

    {
        let mut ctx = run_state.context.write().unwrap();
        ctx.set_play(play);
        if play.ssh_user.is_some() {
            ctx.set_ssh_user(&play.ssh_user.as_ref().unwrap());
        }
        if play.ssh_port.is_some() {
            ctx.set_ssh_port(play.ssh_port.unwrap());
        }
        ctx.unset_role();
    }
    run_state.visitor.read().unwrap().on_play_start(&run_state.context);

    validate_groups(run_state, play)?;
    
    let hosts = get_play_hosts(run_state, play);

    validate_hosts(run_state, play, &hosts)?;
    load_vars_into_context(run_state, play)?;
    //load_external_modules(run_state, play)?;

    // support for serialization if using push configuration
    let batch_size_num = play.batch_size.unwrap_or(0);
    let (_batch_size, batch_count, batches) = get_host_batches(run_state, play, batch_size_num, hosts);

    let mut failed: bool = false;
    let mut failure_message: String = String::new();

    // process each batch task/handlers seperately
    for batch_num in 0..batch_count {
        if failed {
            break;
        }
        let hosts = batches.get(&batch_num).unwrap();
        run_state.visitor.read().unwrap().on_batch(batch_num, batch_count, hosts.len());
        match handle_batch(run_state, play, hosts) {
            Ok(_) => {},
            Err(s) => {
                failed = true;
                failure_message.clear();
                failure_message.push_str(&s.clone());
            }
        }
        run_state.context.read().unwrap().connection_cache.write().unwrap().clear();
    }
    
    // we're done, generate our summary/report & output regardless of failures
    run_state.visitor.read().unwrap().on_play_stop(&run_state.context, failed);
    
    if failed {
        return Err(failure_message.clone());
    } else {
        return Ok(())
    }
}

fn handle_batch(run_state: &Arc<RunState>, play: &Play, hosts: &Vec<Arc<RwLock<Host>>>) -> Result<(), String> {

    // assign the batch
    { let mut ctx = run_state.context.write().unwrap(); ctx.set_targetted_hosts(&hosts); }

    // handle role tasks
    if play.roles.is_some() {
        let roles = play.roles.as_ref().unwrap();
        for invocation in roles.iter() { process_role(run_state, &play, &invocation, HandlerMode::NormalTasks)?; }
    }
    { let mut ctx = run_state.context.write().unwrap(); ctx.unset_role(); }

    // handle loose play tasks
    if play.tasks.is_some() {
        let tasks = play.tasks.as_ref().unwrap();
        for task in tasks.iter() { process_task(run_state, &play, &task, HandlerMode::NormalTasks, None)?; }
    }

    // handle role handlers
    if play.roles.is_some() {
        let roles = play.roles.as_ref().unwrap();
        for invocation in roles.iter() { process_role(run_state, &play, &invocation, HandlerMode::Handlers)?; }
    }   
    { let mut ctx = run_state.context.write().unwrap(); ctx.unset_role(); }  

    // handle loose play handlers
    if play.handlers.is_some() {
        let handlers = play.handlers.as_ref().unwrap();
        for handler in handlers { process_task(run_state, &play, &handler, HandlerMode::Handlers, None)?;  }
    }
    return Ok(())

}

fn check_tags(run_state: &Arc<RunState>, task: &Task, role_invocation: Option<&RoleInvocation>) -> bool {
    match &run_state.tags {
        Some(cli_tags) => {
            match task.get_with() {
                Some(task_with) => match task_with.tags {
                    Some(task_tags) => {
                        for x in task_tags.iter() {  if cli_tags.contains(&x) { return true; } }
                    },
                    None => {}
                },
                None => {}
            };
            match role_invocation {
                Some(role_invoke) => match &role_invoke.tags {
                    Some(role_tags) => {
                        for x in role_tags.iter() { if cli_tags.contains(&x) { return true; } }
                    },
                    None => {}
                },
                None => {}
            };
        }
        None => { return true; }
    }
    return false;
}

fn process_task(run_state: &Arc<RunState>, play: &Play, task: &Task, are_handlers: HandlerMode, role_invocation: Option<&RoleInvocation>) -> Result<(), String> {

    let hosts : HashMap<String, Arc<RwLock<Host>>> = run_state.context.read().unwrap().get_remaining_hosts();
    if hosts.len() == 0 { return Err(String::from("no hosts remaining")) }

    let should_run = check_tags(run_state, task, role_invocation);
    if should_run {
        run_state.context.write().unwrap().set_task(&task);
        run_state.visitor.read().unwrap().on_task_start(&run_state.context, are_handlers);
        run_state.context.write().unwrap().increment_task_count();
        fsm_run_task(run_state, play, task, are_handlers)?;
    }

    return Ok(());
}

fn process_role(run_state: &Arc<RunState>, play: &Play, invocation: &RoleInvocation, are_handlers: HandlerMode) -> Result<(), String> {

    let role_name = invocation.role.clone();
    let (role, role_path) = find_role(run_state, &play, role_name.clone())?;
    {
        let mut ctx = run_state.context.write().unwrap();
        let str_path = directory_as_string(&role_path);
        ctx.set_role(&role, invocation, &str_path);
        if are_handlers == HandlerMode::NormalTasks {
            ctx.increment_role_count();
        }
    }
    run_state.visitor.read().unwrap().on_role_start(&run_state.context);


    let files = match are_handlers {
        HandlerMode::NormalTasks => role.tasks,
        HandlerMode::Handlers    => role.handlers
    };

    if files.is_some() {
        
        let p1 = env::current_dir().expect("could not get current directory");
        let previous = p1.as_path();
        //let role_dirname = directory_as_string(&role_path);
        //let role_dir = Path::new(&role_dirname);
        
        match env::set_current_dir(&role_path) {
            Ok(_) => {}, Err(s) => { return Err(format!("could not chdir into role directory {:?}, {}", role_path, s)) }
        }

        for task_file in files.unwrap().iter() {


            let task_buf = match task_file.starts_with("/") {
                true => {
                    Path::new(task_file).to_path_buf()
                }
                false => {
                    let mut pb = PathBuf::new();
                    pb.push(role_path.clone());
                    match are_handlers {
                        HandlerMode::NormalTasks => { pb.push("tasks"); },
                        HandlerMode::Handlers    => { pb.push("handlers"); },
                    };
                    pb.push(task_file);
                    pb
                }
            };

            let task_fh = jet_file_open(&task_buf.as_path())?;
            let parsed: Result<Vec<Task>, serde_yaml::Error> = serde_yaml::from_reader(task_fh);
            if parsed.is_err() {
                show_yaml_error_in_context(&parsed.unwrap_err(), &task_buf.as_path());
                return Err(format!("edit the file and try again?"));
            }   
            let tasks = parsed.unwrap();
            for task in tasks.iter() {
                process_task(run_state, &play, &task, are_handlers, Some(invocation))?;
            }
        }

        match env::set_current_dir(&previous) {
            Ok(_) => {}, Err(s) => { return Err(format!("could not restore previous directory after role evaluation: {:?}, {}", previous, s)) }
        }

    }

    run_state.visitor.read().unwrap().on_role_stop(&run_state.context);
    return Ok(())

}

// FIXME: this is not yet implemented to make --batch-size work

fn get_host_batches(_run_state: &Arc<RunState>, _play: &Play, _batch_size: usize, hosts: Vec<Arc<RwLock<Host>>>) 
    -> (usize, usize, HashMap<usize, Vec<Arc<RwLock<Host>>>>) {

    let mut results : HashMap<usize, Vec<Arc<RwLock<Host>>>> = HashMap::new();
    //for host in hosts.iter() {
    //    results.insert(0usize, Arc::clone(&host));
    // }

    results.insert(0, hosts.iter().map(|v| Arc::clone(&v)).collect());
    return (1, 1, results);

}

fn get_play_hosts(run_state: &Arc<RunState>,play: &Play) -> Vec<Arc<RwLock<Host>>> {
    
    let groups = &play.groups;
    let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
    
    let has_group_limits = match run_state.limit_groups.len() {
        0 => false,
        _ => true
    };
    let has_host_limits = match run_state.limit_hosts.len() {
        0 => false,
        _ => true
    };

    for group in groups.iter() {

        let group_object = run_state.inventory.read().unwrap().get_group(&group.clone());
        let hosts = group_object.read().unwrap().get_descendant_hosts();
        for (k,v) in hosts.iter() {
            

            if has_host_limits && ! run_state.limit_hosts.contains(k) {
                continue;
            }
            
            if has_group_limits {
                let mut ok = false;
                for group_name in run_state.limit_groups.iter() {
                    if v.read().unwrap().has_group(group_name) {
                        ok = true; 
                        break;
                    }
                }
                if ok {
                    results.insert(k.clone(), Arc::clone(&v));
                }
            } 
            else {
                results.insert(k.clone(), Arc::clone(&v));
            }

        }
    }

    return results.iter().map(|(_k,v)| Arc::clone(&v)).collect();
}

fn validate_groups(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {
    let groups = &play.groups;
    let inv = run_state.inventory.read().unwrap();
    for group_name in groups.iter() {
        if !inv.has_group(&group_name.clone()) {
            return Err(format!("at least one referenced group ({}) is not found in inventory", group_name));
        }
    }
    return Ok(());
}

fn validate_hosts(_run_state: &Arc<RunState>, _play: &Play, hosts: &Vec<Arc<RwLock<Host>>>) -> Result<(), String> {
    if hosts.is_empty() {
        return Err(String::from("no hosts selected by groups in play"));
    }
    return Ok(());
}

fn load_vars_into_context(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {

    let ctx = run_state.context.write().unwrap();
    let mut ctx_vars_storage = serde_yaml::Value::from(serde_yaml::Mapping::new());
    let mut ctx_defaults_storage = serde_yaml::Value::from(serde_yaml::Mapping::new());
    
    if play.vars.is_some() {
        let vars = play.vars.as_ref().unwrap();
        blend_variables(&mut ctx_vars_storage, serde_yaml::Value::Mapping(vars.clone()));
    }

    if play.vars_files.is_some() {
        let vars_files = play.vars_files.as_ref().unwrap();
        for pathname in vars_files {
            let path = Path::new(&pathname);
            let vars_file = jet_file_open(&path)?;
            let parsed: Result<serde_yaml::Mapping, serde_yaml::Error> = serde_yaml::from_reader(vars_file);
            if parsed.is_err() {
                show_yaml_error_in_context(&parsed.unwrap_err(), &path);
                return Err(format!("edit the file and try again?"));
            }
            blend_variables(&mut ctx_vars_storage, serde_yaml::Value::Mapping(parsed.unwrap()));
        }
    }

    if play.defaults.is_some() {
        let defaults = play.defaults.as_ref().unwrap();
        blend_variables(&mut ctx_defaults_storage, serde_yaml::Value::Mapping(defaults.clone()));
    }

    match ctx_vars_storage {
        serde_yaml::Value::Mapping(x) => { *ctx.vars_storage.write().unwrap() = x },
        _ => panic!("unexpected, get_blended_variables produced a non-mapping (1)")
    }
    match ctx_defaults_storage {
        serde_yaml::Value::Mapping(x) => { *ctx.defaults_storage.write().unwrap() = x },
        _ => panic!("unexpected, get_blended_variables produced a non-mapping (1)")
    }

    return Ok(());
}

fn find_role(run_state: &Arc<RunState>, _play: &Play, role_name: String) -> Result<(Role,PathBuf), String> {
    for path_buf in run_state.role_paths.read().unwrap().iter() {
        let mut pb = path_buf.clone();
        pb.push(role_name.clone());
        let mut pb2 = pb.clone();
        pb2.push("role.yml");

        if pb2.exists() {
            let path = pb2.as_path();
            let role_file = jet_file_open(&path)?;
            let parsed: Result<Role, serde_yaml::Error> = serde_yaml::from_reader(role_file);
            if parsed.is_err() {
                show_yaml_error_in_context(&parsed.unwrap_err(), &path);
                return Err(format!("edit the file and try again?"));
            }   
            let role = parsed.unwrap();
            return Ok((role,pb));
        }
    }
    return Err(format!("role not found: {}", role_name));
}  

