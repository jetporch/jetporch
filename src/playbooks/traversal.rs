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
use crate::module_base::list::Task;
use crate::inventory::groups::{has_group,get_group_descendant_hosts};
use crate::util::data::{deduplicate};
use crate::util::io::{directory_as_string,path_as_string,path_basename_as_string};
use std::sync::Mutex;
use std::sync::Arc;

// FIXME: this will take a lot of callback params most likely

/*
#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Play {
    pub jet : JetHeader,
    pub groups: Vec<String>,
    pub roles : Option<Vec<String>>,
    pub force_vars: Option<HashMap<String,Value>>,
    pub defaults: Option<HashMap<String,Value>>,
    pub remote_user: Option<String>,
    pub tasks: Option<Vec<Task>>,
    pub handlers: Option<Vec<Task>>
}
*/

pub struct PlaybookContext {
    pub playbook_path: Arc<Mutex<Option<String>>>,
    pub playbook_directory: Arc<Mutex<Option<String>>>,
    pub play: Arc<Mutex<Option<Play>>>,
    pub task: Arc<Mutex<Option<Task>>>,
    pub host: Arc<Mutex<Option<String>>>,
    pub all_hosts: Arc<Mutex<Option<Vec<String>>>>,
    pub role_path: Arc<Mutex<Option<String>>>,
    pub role_name: Arc<Mutex<Option<String>>>
}
// get_mut
// load(&self, order: Ordering) -> T
// store(&self, val: T, order: Ordering)
// Ordering::Relaxed


impl PlaybookContext {

    pub fn new() -> Self {
        Self {
            playbook_path: Arc::new(Mutex::new(None)),
            playbook_directory: Arc::new(Mutex::new(None)),
            play: Arc::new(Mutex::new(None)),
            task: Arc::new(Mutex::new(None)),
            host: Arc::new(Mutex::new(None)),
            all_hosts: Arc::new(Mutex::new(None)),
            role_path: Arc::new(Mutex::new(None)),
            role_name: Arc::new(Mutex::new(None))
        }
    }

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        *self.playbook_path.lock().unwrap() = Some(path_as_string(&path));
        *self.playbook_directory.lock().unwrap() = Some(directory_as_string(&path));
    }

    pub fn set_play(&mut self, play: Play) {
        *self.play.lock().unwrap() = Some(play);
    }
}

// default implementation mostly just runs the syntax scan
// FIXME: since these share a lot of output in common, what if we construct this
// to take another class as a parameter and then loop over that vector of embedded handlers?

pub trait PlaybookVisitor {

    fn on_playbook_start(&self, context: &PlaybookContext) {
        let path = context.playbook_path.lock().unwrap().unwrap();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &PlaybookContext) {
        let play = context.play.lock().unwrap().unwrap();
        println!("> play start: {}", play.name);
    }

    fn on_play_complete(&self, context: &PlaybookContext) {
        let play = context.play.lock().unwrap().unwrap();
        println!("> play complete: {}", play.name);
    }

    fn on_task_start(&self, context: &PlaybookContext) {
        let task = context.task.unwrap();
        if task.name.is_empty() {
            println!("> task start: {}", task.module);
        } else {
            println!("> task start: {}", task.name);
        }
    }

    fn on_task_complete(&self, context: &PlaybookContext) {
        let task = context.task.lock().unwrap();
        if task.name.is_empty() {
            println!("> task complete: {}", task.module);
        } else {
            println!("> task complete: {}", task.name);
        }
    }

    

    // TODO: functions for loading in variables and such.

}


pub fn playbook_traversal(playbook_paths: &Vec<PathBuf>, context: &PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String> {

    for playbook_path in playbook_paths {

        //context.playbook_path.store(&playbook_path, Ordering::Relaxed);
        visitor.on_playbook_start(&context);

        let playbook_file = jet_file_open(&playbook_path)?;
        let parsed: Result<Vec<Play>, serde_yaml::Error> = serde_yaml::from_reader(playbook_file);

        if parsed.is_err() {
            show_yaml_error_in_context(&parsed.unwrap_err(), &playbook_path);
            return Err(format!("edit the file and try again?"));
        }   
        let play_vec: Vec<Play> = parsed.unwrap();

        for play in play_vec.iter() {

            context.set_play(play);
            visitor.on_play_start(&context);
            validate_jet_version(&play.jet.version)?;
            validate_groups(&play.groups)?;
            validate_hosts(&play.groups)?;
            process_force_vars(&context, &visitor);
            process_remote_user(&context, &visitor);

            if play.roles.is_some() {
                for role_name in play.roles.unwrap().iter() {
                    let role_path = find_role(role_name)?;
                    let defaults_path = role_path.append("defaults");
                    let modules_path = role_path.append("jet_modules");
                    let tasks_path = role_path.append("tasks");
                    traverse_defaults_directory(&context, &visitor, defaults_path)?;
                    traverse_modules_directory(&context, &visitor, modules_path)?;
                    traverse_tasks_directory(&context, &visitor, tasks_path, false)?;
                }
            }

            if play.tasks.is_some() {
                for task in play.tasks { process_task(&context, &visitor, task, false)?; }
            }

            if play.roles.is_some() {
                for role_name in play.roles.unwrap().iter() {
                    let role_path = find_role(role_name)?;
                    let handlers_path = role_path.append("handlers");
                    traverse_tasks_directory(&context, &visitor, handlers_path, true)?;
                }
            }            

            if play.handlers.is_some() {
                for handler in play.handlers {  process_task(&context, &visitor, handler, true); }
            }
            println!("version: {}", &play.jet.version);
            visitor.on_play_complete(&context);
        }
    }
    return Ok(())
}

fn validate_jet_version(version: &String) -> Result<(), String> {
    // FIXME: not implemented
}

fn get_all_hosts(groups: &Vec<String>) {
    let mut results: String = Vec::new();
    for group in groups.iter() {
        let mut hosts = get_group_descendant_hosts(group);
        results.append(&mut hosts);
    }
    return deduplicate(results);
}

fn validate_groups(groups: &Vec<String>) -> Result<(), String> {
    for group in groups.iter() {
        if !has_group(group) {
            return Err(&format!("group ({}) not found", group))
        }
    }
    return Ok();
}

fn validate_hosts(hosts: &Vec<String>) -> Result<(), String> {
    if hosts.is_empty() {
        return Err(String::from("no hosts selected in play"));
    }
    return Ok();
}

fn find_role(rolename: &String) -> Result<(), String> {
    return Err(String::from("find role path is not implemented"));
}  

fn traverse_defaults_directory(context: &PlaybookContext, visitor: &dyn PlaybookVisitor, defaults_path: &PathBuf) -> Result<(), String> {
    return Err(String::from("nope"));
}

fn traverse_modules_directory(context: &PlaybookContext, visitor: &dyn PlaybookVisitor, modules_path: &PathBuf) -> Result<(), String> {
    return Err(String::from("nope"));
}

fn traverse_tasks_directory(context: &PlaybookContext, visitor: &dyn PlaybookVisitor, tasks_path: &PathBuf, are_handlers: bool) -> Result<(), String> {
    return Err(String::from("nope"));
}

fn process_task(context: &PlaybookContext, visitor: &dyn PlaybookVisitor, task: &Task, are_handlers: bool) -> Result<(), String> {
    return Err(String::from("nope"));
}

fn process_force_vars(context: &PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String>  {
    return Err(String::from("nope"));
}

fn process_remote_user(context: &PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String>  {
    return Err(String::from("nope"));

}