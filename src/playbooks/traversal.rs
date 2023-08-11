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

use crate::playbooks::language::{Play};
use std::path::PathBuf;
use crate::util::io::jet_file_open;
use crate::util::yaml::show_yaml_error_in_context;
//use crate::module_base::list::Task;
use crate::inventory::groups::{has_group,get_group_descendant_hosts};
use crate::util::data::{deduplicate};
use crate::util::io::{directory_as_string,path_as_string};
use crate::playbooks::visitor::PlaybookVisitor;
use crate::playbooks::context::PlaybookContext;
use std::collections::HashMap;
use crate::connection::factory::ConnectionFactory;
use serde_yaml::Value;
use crate::module_base::common::IsTask;
use crate::module_base::list::Task;

// ============================================================================
// PUBLIC API, see syntax.rs/etc for usage
// ============================================================================

pub fn playbook_traversal(playbook_paths: &Vec<PathBuf>, 
    context: &mut PlaybookContext, 
    visitor: &dyn PlaybookVisitor,
    connection_factory: &dyn ConnectionFactory,
    batch_size: Option<usize>) -> Result<(), String> {

    let batch_size_num: usize = match batch_size {
        Some(x) => x,
        None => 0
    };
        
    for playbook_path in playbook_paths {

        context.set_playbook_path(playbook_path);
        visitor.on_playbook_start(&context);

        let playbook_file = jet_file_open(&playbook_path)?;
        let parsed: Result<Vec<Play>, serde_yaml::Error> = serde_yaml::from_reader(playbook_file);
       
        if parsed.is_err() {
            show_yaml_error_in_context(&parsed.unwrap_err(), &playbook_path);
            return Err(format!("edit the file and try again?"));
        }   

        let plays: Vec<Play> = parsed.unwrap();
        for play in plays.iter() {

            context.set_play(&play);
            context.set_remote_user(&play);

            context.unset_role();
            visitor.on_play_start(&context);

            // FIXME: add teh concept of host_sets here, and then loop over the sets
            // for use with batch... everything goes an indent level deeper, basically.

            validate_jet_version(&context, &play.jet.version)?;
            validate_groups(&context, &play.groups)?;
            validate_hosts(&context, &play.groups)?;

            let hosts = get_all_hosts(&context, &play.groups);
            let (batch_size, batches) = get_host_batches(batch_size_num, hosts);

            for batch_num in 1..batch_size {

                let hosts = batches.get(&batch_num).unwrap();

                // FIXME: does the context even need to know?
                // context.set_hosts(hosts);

                process_force_vars(&context, visitor, play.force_vars);
                register_external_modules(&context, visitor)?;

                if play.roles.is_some() {
                    let roles = play.roles.as_ref().unwrap();
                    for role in roles.iter() {
                        let role_name = role.name.clone();
                        let role_path = find_role(&context, visitor, role_name.clone())?;
                        let pathbuf = role_path.to_path_buf();
                        context.set_role(role.name.clone(), directory_as_string(&pathbuf));
                        apply_defaults_directory(&context, visitor)?;
                        register_external_modules(&context, visitor)?;
                        traverse_tasks_directory(&context, visitor, connection_factory, false)?;
                    }
                }
                context.unset_role();

                if play.tasks.is_some() {
                    let tasks = play.tasks.as_ref().unwrap();
                    for task in tasks.iter() { 
                        process_task(&context, visitor, connection_factory, task, false)?; 
                    }
                }

                if play.roles.is_some() {
                    let roles = play.roles.as_ref().unwrap();
                    for role in roles.iter() {
                        let role_name = role.name.clone();
                        let role_path = find_role(&context, visitor, role_name.clone())?;                        
                        let pathbuf = role_path.to_path_buf();
                        context.set_role(role.name.clone(), directory_as_string(&pathbuf));
                        traverse_tasks_directory(&context, visitor, connection_factory, true)?;
                    }
                }     
                context.unset_role();      

                if play.handlers.is_some() {
                    let handlers = play.handlers.as_ref().unwrap();
                    for handler in handlers {  
                        process_task(&context, visitor, connection_factory, handler, true)?; 
                    }
                }
            }
            println!("version: {}", &play.jet.version);
            visitor.on_play_complete(&context);
        }
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
                    /*
                    let mut defaults_path = role_path.clone();
                    let mut module_path = role_path.clone();
                    let mut tasks_path = role_path.clone();
                    defaults_path.push("defaults");
                    module_path.push("jet_modules");
                    */

fn validate_jet_version(_context: &PlaybookContext, version: &String) -> Result<(), String> {
    // FIXME
    if version.eq("5000") {
        return Err(format!("the version cannot be 5000, was: {}", version));
    }
    return Ok(());
}

fn get_all_hosts(_context: &PlaybookContext, groups: &Vec<String>) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for group in groups.iter() {
        let mut hosts = get_group_descendant_hosts(group.clone());
        results.append(&mut hosts);
    }
    return deduplicate(results);
}

fn validate_groups(_context: &PlaybookContext, groups: &Vec<String>) -> Result<(), String> {
    for group in groups.iter() {
        if !has_group(group.clone()) {
            return Err(format!("referenced group ({}) not found in inventory", group));
        }
    }
    return Ok(());
}

fn validate_hosts(_context: &PlaybookContext, hosts: &Vec<String>) -> Result<(), String> {
    if hosts.is_empty() {
        return Err(String::from("no hosts selected by groups in play"));
    }
    return Ok(());
}

fn find_role(context: &PlaybookContext, visitor: &dyn PlaybookVisitor, rolename: String) -> Result<PathBuf, String> {
    // FIXME
    visitor.debug(String::from("finding role..."));

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


fn apply_defaults_directory(context: &PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String> {
    // FIXME
    // all the files in the defaults directory should be loaded as YAML files and then send to the context
    // easiest if we just use the blend functions
    visitor.debug(String::from("loading defaults directory"));
    return Ok(());
}

fn register_external_modules(context: &PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String> {
    // FIXME
    // if there are any files found in the modules directory add their module names to the modules registry in the context
    // object.  We should look in context.role_paths directories as well as if there is a module path parameter
    // in the CLI, which there is currently not.  Also pbdir/jet_modules
    // then the "External" module code can  use this to find the actual module.  What we should register is the full
    // path

    visitor.debug(String::from("loading modules directory"));
    return Ok(());
}

fn traverse_tasks_directory(context: &PlaybookContext, 
    visitor: &dyn PlaybookVisitor, 
    connection_factory: &dyn ConnectionFactory, 
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
    visitor.debug(String::from("loading tasks directory"));
    return Ok(());

}

fn process_task(context: &PlaybookContext, 
    visitor: &dyn PlaybookVisitor, 
    connection_factory: &dyn ConnectionFactory, 
    task: &Task,
    are_handlers: bool) -> Result<(), String> {

       
    // ask the connection factory for a connection, call the global module 'workflow'
    // code (TBD) and record the result from that workflow via the context
    // we don't actually crash here.  The workflow code should handle parallelization
    // as this function needs access to the task list.


    visitor.debug(String::from("processing task"));

    // use context.get_hosts for what hosts to talk to.

    return Ok(());

}

fn process_force_vars(context: &PlaybookContext, 
    visitor: &dyn PlaybookVisitor, 
    force_vars: Option<HashMap<std::string::String, Value>>) -> Result<(), String>  {
    
    // FIXME
    visitor.debug(String::from("processing force_vars"));
    return Ok(());

}

