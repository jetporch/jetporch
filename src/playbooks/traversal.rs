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
use crate::playbooks::language::Role;
use crate::connection::factory::ConnectionFactory;
use crate::registry::list::Task;
use crate::runner::task_fsm::fsm_run_task;
use crate::inventory::inventory::Inventory;
use crate::inventory::hosts::Host;
use crate::util::io::{jet_file_open,directory_as_string};
use crate::util::yaml::{blend_variables,show_yaml_error_in_context};
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc,RwLock};
use std::path::Path;
//use std::ops::Deref;
use std::env;

// traversal code walks a playbook and does "things" to it, different behaviors
// are available, see cli/playbooks.rs.  The RunState encapsulates common
// parameters to avoid functions taking extremely large signatures, but it should
// not be thought of as a pseudo-global, for instance modules are eventually
// only given access to the context and visitor portions.

pub struct RunState {
    pub inventory: Arc<RwLock<Inventory>>,
    pub playbook_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub context: Arc<RwLock<PlaybookContext>>,
    pub visitor: Arc<RwLock<dyn PlaybookVisitor>>,
    pub connection_factory: Arc<RwLock<dyn ConnectionFactory>>,
    pub default_user: Option<String>
}

// ============================================================================
// PUBLIC API, see syntax.rs/etc for usage
// ============================================================================

pub fn playbook_traversal(run_state: &Arc<RunState>) -> Result<(), String> {
        
    for playbook_path in run_state.playbook_paths.read().unwrap().iter() {

        { 
            let mut ctx = run_state.context.write().unwrap(); 
            ctx.set_playbook_path(playbook_path); 
            ctx.set_default_remote_user(run_state.default_user.clone());
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
        

        //println!("PP: {}", playbook_path.display());
        let pbdirname = directory_as_string(playbook_path);
        //println!("dirname: {}", pbdirname);
        let pbdir = Path::new(&pbdirname);
        if pbdirname.eq(&String::from("")) {
        } else {
            env::set_current_dir(&pbdir).expect("could not chdir into playbook directory");
        }

        let plays: Vec<Play> = parsed.unwrap();
        for play in plays.iter() {
            match handle_play(&run_state, play) {
                Ok(_) => {},
                Err(s) => { break; }
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

// ==============================================================================

fn handle_play(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {

    // configure the current playbook context
    {
        let mut ctx = run_state.context.write().unwrap();
        ctx.set_play(play);
        // FIXME: this shouldn't be set here
        //ctx.set_remote_user(&play, run_state.default_user.clone());
        ctx.unset_role();
    }
    run_state.visitor.read().unwrap().on_play_start(&run_state.context);

    // load the basic setup details before dealing with tasks
    //load_play_vars(run_state, play)?;
    //load_play_vars_files(run_state, play)?;
    validate_groups(run_state, play)?;
    let hosts = get_play_hosts(run_state, play);
    validate_hosts(run_state, play, &hosts)?;
    load_vars_into_context(run_state, play);
    load_external_modules(run_state, play)?;

    // support for serialization of push configuration
    // in non-push modes the batch size is irrelevant
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

// ==============================================================================

fn handle_batch(run_state: &Arc<RunState>, play: &Play, hosts: &Vec<Arc<RwLock<Host>>>) -> Result<(), String> {

    // assign the batch
    { let mut ctx = run_state.context.write().unwrap(); ctx.set_targetted_hosts(&hosts); }

    // handle role tasks
    if play.roles.is_some() {
        let roles = play.roles.as_ref().unwrap();
        for role in roles.iter() { process_role(run_state, &play, &role, false)?; }
    }
    { let mut ctx = run_state.context.write().unwrap(); ctx.unset_role(); }

    // handle loose play tasks
    if play.tasks.is_some() {
        let tasks = play.tasks.as_ref().unwrap();
        for task in tasks.iter() { process_task(run_state, &play, &task, false)?; }
    }

    // handle role handlers
    if play.roles.is_some() {
        let roles = play.roles.as_ref().unwrap();
        for role in roles.iter() { process_role(run_state, &play, &role, true)?; }
    }   
    { let mut ctx = run_state.context.write().unwrap(); ctx.unset_role(); }  

    // handle loose play handlers
    if play.handlers.is_some() {
        let handlers = play.handlers.as_ref().unwrap();
        for handler in handlers { process_task(run_state, &play, &handler, true)?;  }
    }
    return Ok(())

}

// ==============================================================================

fn process_task(run_state: &Arc<RunState>, _play: &Play, task: &Task, are_handlers: bool) -> Result<(), String> {

    run_state.context.write().unwrap().set_task(&task);
    run_state.visitor.read().unwrap().on_task_start(&run_state.context);
    run_state.context.write().unwrap().increment_task_count();

    // launch a task, possibly using the thread pool
    // this method contains all logic for module dispatch and result handling

    if !run_state.visitor.read().unwrap().is_syntax_only() {
        fsm_run_task(run_state, task, are_handlers)?;
    }

    return Ok(());
}


// ==============================================================================

fn process_role(run_state: &Arc<RunState>, play: &Play, role: &Role, are_handlers: bool) -> Result<(), String> {

    // pre-process role loads all variables and module paths into
    // the context.  It does not execute any tasks.

    let role_name = role.name.clone();
    let role_path = find_role(run_state, &play, role_name.clone())?;

    let pathbuf = role_path.to_path_buf();
    // TODO: FIXME: role parameters also need to be set into the context
    // TODO: FIXME: blending functions need to take the context as a parameter
    {
        let mut ctx = run_state.context.write().unwrap();
        ctx.set_role(role.name.clone(), directory_as_string(&pathbuf));
    }
    run_state.visitor.read().unwrap().on_role_start(&run_state.context);
    load_defaults_directory_for_role(run_state, &play)?;
    load_external_modules(run_state, &play)?;
    load_tasks_directory_for_role(run_state, &play, are_handlers)?;

    run_state.visitor.read().unwrap().on_role_stop(&run_state.context);
    return Ok(())

}

// ============================================================================

fn get_host_batches(_run_state: &Arc<RunState>, _play: &Play, _batch_size: usize, hosts: Vec<Arc<RwLock<Host>>>) 
    -> (usize, usize, HashMap<usize, Vec<Arc<RwLock<Host>>>>) {

    // partition a list of hosts into a number of batches.
    // FIXME: not implemented!

    let mut results : HashMap<usize, Vec<Arc<RwLock<Host>>>> = HashMap::new();
    //for host in hosts.iter() {
    //    results.insert(0usize, Arc::clone(&host));
    // }

    results.insert(0, hosts.iter().map(|v| Arc::clone(&v)).collect());
    return (1, 1, results);

}

// ==============================================================================

fn get_play_hosts(run_state: &Arc<RunState>,play: &Play) -> Vec<Arc<RwLock<Host>>> {

    // given a list of group names, return a vector of all hosts selected by those groups.
    let groups = &play.groups;
    let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
    for group in groups.iter() {
        let group_object = run_state.inventory.read().unwrap().get_group(&group.clone());
        let hosts = group_object.read().unwrap().get_descendant_hosts();
        for (k,v) in hosts.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
    }
    return results.iter().map(|(_k,v)| Arc::clone(&v)).collect();
}

// ==============================================================================

fn validate_groups(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {

    // given a list of group names verify at least one of the group names is actually present
    // in inventory.  

    // FIXME: warn if any group names are not found in inventory if this does not produce
    // an error.
    let groups = &play.groups;
    let inv = run_state.inventory.read().unwrap();
    for group_name in groups.iter() {
        if !inv.has_group(&group_name.clone()) {
            return Err(format!("at least one referenced group ({}) is not found in inventory", group_name));
        }
    }
    return Ok(());
}

// ==============================================================================

fn validate_hosts(_run_state: &Arc<RunState>, _play: &Play, hosts: &Vec<Arc<RwLock<Host>>>) -> Result<(), String> {
    
    // given a list of a hosts, verify there is at least one host, otherwise the play should fail
    
    if hosts.is_empty() {
        return Err(String::from("no hosts selected by groups in play"));
    }
    return Ok(());
}

// ==============================================================================

fn load_vars_into_context(run_state: &Arc<RunState>, play: &Play) -> Result<(), String> {

    let ctx = run_state.context.write().unwrap();

    // BOOKMARK ------ VVVVVVVVVVVVVVVV ---- FIXME
    // FIXME: use these - not the context versions - and set into context at the end.

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
            let vars_file_parse_result: Result<serde_yaml::Mapping, serde_yaml::Error> = serde_yaml::from_reader(vars_file);
            if vars_file_parse_result.is_err() {
                show_yaml_error_in_context(&vars_file_parse_result.unwrap_err(), &path);
                return Err(format!("edit the file and try again?"));
            }
            blend_variables(&mut ctx_vars_storage, serde_yaml::Value::Mapping(vars_file_parse_result.unwrap()));
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


// ==============================================================================

fn find_role(run_state: &Arc<RunState>, _play: &Play, rolename: String) -> Result<PathBuf, String> {

    // given a role name, return the location...
    // FIXME: not sure why the tuple is in the return type here

    run_state.visitor.read().unwrap().debug(&String::from("finding role..."));

    // look in context.get_roles_paths() and also in "./roles" (that first) for a role with the right name.
    // if not found, raise an error
    // then raise an error if there is not one of the valid roles subdirectories:
    //     tasks defaults files templates jet_modules handlers

    /*
        path_walk(path, |groups_file_path| {
        let group_name = path_basename_as_string(&groups_file_path).clone();
        let groups_file = jet_file_open(&groups_file_path)?;
        let groups_file_parse_result: Result<YamlGroup, serde_yaml::Error> = serde_yaml::from_reader(groups_file);
    */


    return Err(String::from("find role path is not implemented"));
}  

// ==============================================================================


fn load_defaults_directory_for_role(_run_state: &Arc<RunState>, _play: &Play) -> Result<(), String> {

    // loads a defaults directory into context
    // TODO: check the context, we a role might be involved, in which case, look there also

    // FIXME
    // all the files in the defaults directory should be loaded as YAML files and then send to the context
    // easiest if we just use the blend functions
    //visitor.read().unwrap().debug(String::from("loading defaults directory"));
    return Ok(());
}

// ==============================================================================

fn load_external_modules(_run_state: &Arc<RunState>, _play: &Play) -> Result<(), String> {

    // makes an external module findable by adding to the search path
    // TODO: check the context, if we are in a role the path is different

    // FIXME
    // if there are any files found in the modules directory add their module names to the modules registry in the context
    // object.  We should look in context.role_paths directories as well as if there is a module path parameter
    // in the CLI, which there is currently not.  Also pbdir/jet_modules
    // then the "External" module code can  use this to find the actual module.  What we should register is the full
    // path

    // FIXME: look at the context and see if there is an active role, use an appropriate search path.

    //visitor.read().unwrap().debug(String::from("loading modules directory"));
    return Ok(());
}

// ==============================================================================

fn load_tasks_directory_for_role(_run_state: &Arc<RunState>, _play: &Play, _are_handlers: bool) -> Result<(), String> {

    // this should read main.yaml in the role and run all tasks within it.

        // FIXME:
        // get all YAML files in the task directory, do main.yml first, the context
        // given to the include module should allow it to find things in the right place
        // so technically this is not a full traversal, the Include module must then
        // call the same function this would call on other files with the same
        // parameters.  From Include, we may need to tell the Workflow to not
        // parallelize further.

        // use context.get_hosts for what hosts to talk to.

    // FIXME
    //visitor.read().unwrap().debug(String::from("loading tasks directory"));
    return Ok(());

}

