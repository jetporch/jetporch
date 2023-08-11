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
use crate::util::io::{directory_as_string,path_as_string}; // path_basename_as_string};
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
    pub play: Arc<Mutex<Option<String>>>,
    pub task: Arc<Mutex<Option<String>>>,
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

    pub fn set_play(&mut self, play: &Play) {
        *self.play.lock().unwrap() = Some(play.name.clone());
    }
}

/* MAYBE FOR LATER
use crossbeam_utils::atomic::AtomicCell;
use std::thread;

fn main() {
    let rofl = Some("lol".to_string());
    
    let foo = AtomicCell::new(None);
    foo.store(rofl);
    
    let bar = thread::spawn(move || {
        println!("{:?}", foo.into_inner());
    });
    
    bar.join().unwrap();
}
*/
// default implementation mostly just runs the syntax scan
// FIXME: since these share a lot of output in common, what if we construct this
// to take another class as a parameter and then loop over that vector of embedded handlers?

pub trait PlaybookVisitor {

    fn on_playbook_start(&self, context: &PlaybookContext) {
        let arc = context.playbook_path.lock().unwrap();
        let path = arc.as_ref().unwrap();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &PlaybookContext) {
        let arc = context.play.lock().unwrap();
        let play = arc.as_ref().unwrap();
        println!("> play start: {}", play);
    }

    fn on_play_complete(&self, context: &PlaybookContext) {
        let arc = context.play.lock().unwrap();
        let play = arc.as_ref().unwrap();
        println!("> play complete: {}", play);
    }

    fn on_task_start(&self, context: &PlaybookContext) {
        let arc = context.task.lock().unwrap();
        let task = arc.as_ref().unwrap();
        //let module = task.get_module();
        println!("> task start: {}", task);
    }

    fn on_task_complete(&self, context: &PlaybookContext) {
        let arc = context.task.lock().unwrap();
        let task = arc.as_ref().unwrap();
        println!("> task complete: {}", task);
    }

    

    // TODO: functions for loading in variables and such.

}


pub fn playbook_traversal(playbook_paths: &Vec<PathBuf>, context: &mut PlaybookContext, visitor: &dyn PlaybookVisitor) -> Result<(), String> {

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

            context.set_play(&play);
            visitor.on_play_start(&context);
            validate_jet_version(&play.jet.version)?;
            validate_groups(&play.groups)?;
            validate_hosts(&play.groups)?;
            process_force_vars(&context, visitor);
            process_remote_user(&context, visitor);

            if play.roles.is_some() {
                let roles = play.roles.as_ref().unwrap();
                for role_name in roles.iter() {
                    let role_path : PathBuf = find_role(role_name)?;
                    let mut defaults_path = role_path.clone();
                    let mut module_path = role_path.clone();
                    let mut tasks_path = role_path.clone();
                    defaults_path.push("defaults");
                    module_path.push("jet_modules");
                    tasks_path.push("tasks");
                    traverse_defaults_directory(&context, visitor, &defaults_path)?;
                    traverse_modules_directory(&context, visitor, &module_path)?;
                    traverse_tasks_directory(&context, visitor, &tasks_path, false)?;
                }
            }

            if play.tasks.is_some() {
                let tasks = play.tasks.as_ref().unwrap();
                for task in tasks.iter() { 
                    process_task(&context, visitor, &task, false)?; 
                }
            }

            if play.roles.is_some() {
                let roles = play.roles.as_ref().unwrap();
                for role_name in roles.iter() {
                    let role_path = find_role(role_name)?;
                    let mut handlers_path = role_path.clone();
                    handlers_path.push("handlers");
                    traverse_tasks_directory(&context, visitor, &handlers_path, true)?;
                }
            }            

            if play.handlers.is_some() {
                let handlers = play.handlers.as_ref().unwrap();
                for handler in handlers {  
                    process_task(&context, visitor, &handler, true)?; 
                }
            }
            println!("version: {}", &play.jet.version);
            visitor.on_play_complete(&context);
        }
    }
    return Ok(())
}

fn validate_jet_version(version: &String) -> Result<(), String> {
    return Ok(());
}

fn get_all_hosts(groups: &Vec<String>) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for group in groups.iter() {
        let mut hosts = get_group_descendant_hosts(group.clone());
        results.append(&mut hosts);
    }
    return deduplicate(results);
}

fn validate_groups(groups: &Vec<String>) -> Result<(), String> {
    for group in groups.iter() {
        if !has_group(group.clone()) {
            return Err(format!("group ({}) not found", group))
        }
    }
    return Ok(());
}

fn validate_hosts(hosts: &Vec<String>) -> Result<(), String> {
    if hosts.is_empty() {
        return Err(String::from("no hosts selected in play"));
    }
    return Ok(());
}

fn find_role(rolename: &String) -> Result<PathBuf, String> {
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