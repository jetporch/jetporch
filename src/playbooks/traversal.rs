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
// ABOUT: traversal.rs
// loads playbook YAML and walks the object therein, for usage see commands
// like cli/show.rs
// ===================================================================================

use crate::playbooks::language::{Play};
use crate::util::io::jet_file_open;
use crate::util::io::{directory_as_string};
use crate::util::yaml::show_yaml_error_in_context;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;
use crate::connection::factory::ConnectionFactory;
use crate::registry::list::Task;
use crate::runner::task_fsm::fsm_run_task;
use crate::inventory::inventory::Inventory;
use crate::inventory::hosts::Host;
use std::path::PathBuf;
use serde_yaml::Value;
use std::collections::HashMap;
use std::sync::{Arc,RwLock};

// ============================================================================
// PUBLIC API, see syntax.rs/etc for usage
// ============================================================================

pub fn playbook_traversal(
    inventory: Arc<RwLock<Inventory>>,
    playbook_paths: &Vec<PathBuf>, 
    context: Arc<RwLock<PlaybookContext>>, 
    visitor: Arc<RwLock<dyn PlaybookVisitor>>,
    connection_factory: Arc<RwLock<dyn ConnectionFactory>>,
    default_user: String) -> Result<(), String> {

    // perhaps this should not be a CLI option!
    // FIXME: pass default_user to set_remote_user
        
    for playbook_path in playbook_paths {

        { let mut ctx = context.write().unwrap(); ctx.set_playbook_path(playbook_path); }
        visitor.read().unwrap().on_playbook_start(&context);

        let playbook_file = jet_file_open(&playbook_path)?;
        let parsed: Result<Vec<Play>, serde_yaml::Error> = serde_yaml::from_reader(playbook_file);
       
        if parsed.is_err() {
            show_yaml_error_in_context(&parsed.unwrap_err(), &playbook_path);
            return Err(format!("edit the file and try again?"));
        }   

        let plays: Vec<Play> = parsed.unwrap();
        for play in plays.iter() {


            // FIXME: make this a function!

            let batch_size_num = play.batch_size.unwrap_or(0);

            {
                let mut ctx = context.write().unwrap();
                ctx.set_play(&play);
                ctx.set_remote_user(&play, default_user.clone());
                ctx.unset_role();
            }

            // FIXME: visitor should also be RwLock I think?
            visitor.read().unwrap().on_play_start(&context);

            load_vars(&inventory, &context, &play.vars);
            load_vars_files(&inventory, &context, &play.vars_files);

            // FIXME: add teh concept of host_sets here, and then loop over the sets
            // for use with batch... everything goes an indent level deeper, basically.

            validate_groups(&inventory, &context, &play.groups)?;
            validate_hosts(&inventory, &context, &play.groups)?;

            let hosts = get_all_hosts(&inventory, &context, &play.groups);
            let (batch_size, batches) = get_host_batches(batch_size_num, hosts);
            println!("DEBUG: batch size: {}", batch_size);


            register_external_modules(&context, &visitor)?;

            for batch_num in 0..batch_size {

                let hosts = batches.get(&batch_num).unwrap();


                if play.roles.is_some() {
                    let roles = play.roles.as_ref().unwrap();
                    for role in roles.iter() {
                        let role_name = role.name.clone();
                        let role_path = find_role(&context, &visitor, role_name.clone())?;
                        let pathbuf = role_path.to_path_buf();
                        // FIXME: also set role.params in context
                        // FIXME: blending logic in context
                        {
                            let mut ctx = context.write().unwrap();
                            ctx.set_role(role.name.clone(), directory_as_string(&pathbuf));
                        }
                        visitor.read().unwrap().on_role_start(&context);
                        apply_defaults_directory(&context, &visitor)?;
                        register_external_modules(&context, &visitor)?;
                        load_tasks_directory(&context, &visitor, &connection_factory, false)?;
                        visitor.read().unwrap().on_role_stop(&context);
                    }
                }
                { let mut ctx = context.write().unwrap(); ctx.unset_role(); }

                if play.tasks.is_some() {
                    let tasks = play.tasks.as_ref().unwrap();

                    for task in tasks.iter() {
                        //println!("NAME! {}", task.get_name());

                        //blip(task);

                        //context.set_task(task.get_name().clone());
                        visitor.read().unwrap().on_task_start(&context);
                        
                        process_task(&inventory, &context, &visitor, &connection_factory, task, false)?; 
                        //visitor.on_task_stop(&context);
                        
                    }
                }

                /*
                if play.roles.is_some() {
                    let roles = play.roles.as_ref().unwrap();
                    for role in roles.iter() {
                        context.write().unwrap().increment_task_count();
                        visitor.on_role_start(&context);
                        let role_name = role.name.clone();
                        let role_path = find_role(&context, visitor, role_name.clone())?;                        
                        let pathbuf = role_path.to_path_buf();
                        context.set_role(role.name.clone(), directory_as_string(&pathbuf));
                        load_tasks_directory(&context, visitor, connection_factory, true)?;
                        visitor.on_role_stop(&context);
                    }
                }     
                context.unset_role();      

                if play.handlers.is_some() {
                    let handlers = play.handlers.as_ref().unwrap();
                    for handler in handlers {  
                        context.set_task(handler.get_name().clone());
                        visitor.on_task_start(&context);
                        process_task(&context, visitor, connection_factory, handler, true)?; 
                        visitor.on_task_stop(&context);
                    }
                }
                */
            }
            visitor.read().unwrap().on_play_stop(&context);
        }

        // FIXME: reset roles and task counts, role and play names, in context etc.
    }
    return Ok(())
}

// ============================================================================
// PRIVATE
// ============================================================================


fn get_host_batches(batch_size: usize, hosts: Vec<String>) -> (usize, HashMap<usize, String>) {
    
    // FIXME: implement logic -- right now this means --batch-size is ignored
    // FIXME: update CLI docs that implies batch size can be a list of values, or change the signature
    // to make it work.

    let mut results : HashMap<usize, String> = HashMap::new();
    for host in hosts.iter() {
        results.insert(0usize, host.clone());
    }
    return (1, results);

}

fn get_all_hosts(inventory: &Arc<RwLock<Inventory>>, _context: &Arc<RwLock<PlaybookContext>>, groups: &Vec<String>) -> Vec<String> {
    let inventory = inventory.read().unwrap();
    let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
    for group in groups.iter() {
        let group_object = inventory.get_group(&group.clone());
        let hosts = group_object.read().unwrap().get_descendant_hosts();
        for (k,v) in hosts.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
    }
    return results.iter().map(|(k,v)| k.clone()).collect();
}


fn validate_groups(inventory: &Arc<RwLock<Inventory>>, _context: &Arc<RwLock<PlaybookContext>>, groups: &Vec<String>) -> Result<(), String> {
    let inv = inventory.read().unwrap();
    for group_name in groups.iter() {
        if !inv.has_group(&group_name.clone()) {
            return Err(format!("at least one referenced group ({}) is not found in inventory", group_name));
        }
    }
    return Ok(());
}

fn validate_hosts(inventory: &Arc<RwLock<Inventory>>, _context: &Arc<RwLock<PlaybookContext>>, hosts: &Vec<String>) -> Result<(), String> {
    if hosts.is_empty() {
        return Err(String::from("no hosts selected by groups in play"));
    }
    return Ok(());
}
                    /*
                    let mut defaults_path = role_path.clone();
                    let mut module_path = role_path.clone();
                    let mut tasks_path = role_path.clone();
                    defaults_path.push("defaults");
                    module_path.push("jet_modules");
                    */

fn load_vars(inventory: &Arc<RwLock<Inventory>>, _context: &Arc<RwLock<PlaybookContext>>, _map: &Option<HashMap<String,Value>>) -> Result<(), String> {
    return Err(String::from("not implemented"));

}

fn load_vars_files(inventory: &Arc<RwLock<Inventory>>, _context: &Arc<RwLock<PlaybookContext>>, _list: &Option<Vec<String>>) -> Result<(), String> {
    return Err(String::from("not implemented"));
}

fn find_role(context: &Arc<RwLock<PlaybookContext>>, visitor: &Arc<RwLock<dyn PlaybookVisitor>>, rolename: String) -> Result<PathBuf, String> {
    // FIXME
    visitor.read().unwrap().debug(String::from("finding role..."));

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


fn apply_defaults_directory(context: &Arc<RwLock<PlaybookContext>>, visitor: &Arc<RwLock<dyn PlaybookVisitor>>) -> Result<(), String> {
    // FIXME
    // all the files in the defaults directory should be loaded as YAML files and then send to the context
    // easiest if we just use the blend functions
    visitor.read().unwrap().debug(String::from("loading defaults directory"));
    return Ok(());
}

fn register_external_modules(context: &Arc<RwLock<PlaybookContext>>, visitor: &Arc<RwLock<dyn PlaybookVisitor>>) -> Result<(), String> {
    // FIXME
    // if there are any files found in the modules directory add their module names to the modules registry in the context
    // object.  We should look in context.role_paths directories as well as if there is a module path parameter
    // in the CLI, which there is currently not.  Also pbdir/jet_modules
    // then the "External" module code can  use this to find the actual module.  What we should register is the full
    // path

    visitor.read().unwrap().debug(String::from("loading modules directory"));
    return Ok(());
}

fn load_tasks_directory(context: &Arc<RwLock<PlaybookContext>>, 
    visitor: &Arc<RwLock<dyn PlaybookVisitor>>, 
    connection_factory: &Arc<RwLock<dyn ConnectionFactory>>, 
    are_handlers: bool) -> Result<(), String> {

        // FIXME:
        // get all YAML files in the task directory, do main.yml first, the context
        // given to the include module should allow it to find things in the right place
        // so technically this is not a full traversal, the Include module must then
        // call the same function this would call on other files with the same
        // parameters.  From Include, we may need to tell the Workflow to not
        // parallelize further.

        // use context.get_hosts for what hosts to talk to.

    // FIXME
    visitor.read().unwrap().debug(String::from("loading tasks directory"));
    return Ok(());

}

fn process_task(
    inventory: &Arc<RwLock<Inventory>>, 
    context: &Arc<RwLock<PlaybookContext>>, 
    visitor: &Arc<RwLock<dyn PlaybookVisitor>>, 
    connection_factory: &Arc<RwLock<dyn ConnectionFactory>>, 
    task: &Task,
    are_handlers: bool) -> Result<(), String> {

    
    // ask the connection factory for a connection, call the global module 'workflow'
    // code (TBD) and record the result from that workflow via the context
    // we don't actually crash here.  The workflow code should handle parallelization
    // as this function needs access to the task list.

    context.write().unwrap().increment_task_count();

    visitor.read().unwrap().debug(String::from("processing task"));

    // FIXME: we need some logic to say if are_handlers = true only run the task if
    // the task is modified.

    fsm_run_task(inventory, context, visitor, connection_factory, task)?; 


    // use context.get_hosts for what hosts to talk to.

    return Ok(());

}
