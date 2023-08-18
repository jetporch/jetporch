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
// ABOUT: context.rs
// the playbook context is a struct of status that remains available to all objects
// while walking the playbook tree, it, for instance, allows any task to know the
// current role, what the current path root is, and so on. While parts of the context
// are mutable, they should not be adjusted outside of playbook traversal code.
// ===================================================================================

use crate::util::io::{path_as_string,directory_as_string};
use crate::playbooks::language::Play;
use std::path::PathBuf;
use std::collections::{HashSet,HashMap};
use crate::inventory::hosts::Host;
use std::sync::{Arc,RwLock};

pub struct PlaybookContext {
    
    pub playbook_path: Option<String>,
    pub playbook_directory: Option<String>,
    pub play: Option<String>,
    
    pub role: Option<String>,
    pub role_path: Option<String>,
    pub role_name: Option<String>,
    pub role_count: usize,

    pub task_count: usize,
    pub task: Option<String>,

    // FIXME: should this be here, it should be passed around maybe?
    pub host: Option<String>,
    
    pub all_hosts: HashMap<String, Arc<RwLock<Host>>>,
    pub remaining_hosts: HashMap<String, Arc<RwLock<Host>>>,
    pub failed_hosts: HashMap<String, Arc<RwLock<Host>>>,

    // FIXME: should this be here, it's a property of the connection
    pub remote_user: Option<String>,
}

impl PlaybookContext {

    pub fn new() -> Self {
        Self {
            playbook_path: None,
            playbook_directory: None,
            play: None,
            role: None,
            task: None,
            role_count : 0,
            task_count : 0,
            host: None,
            all_hosts: HashMap::new(),
            remaining_hosts: HashMap::new(),
            failed_hosts: HashMap::new(),
            role_path: None,
            role_name: None,
            remote_user: None
        }
    }

    // ===============================================================================
    // HOST TARGETTING

    // get all selected hosts in the play
    // FIXME: need a method for non-failed hosts
    pub fn get_remaining_hosts(&self) -> HashMap<String, Arc<RwLock<Host>>> {
        let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
        for (k,v) in self.remaining_hosts.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
        return results;
    }

    pub fn set_targetted_hosts(&self, hosts: &Vec<Arc<RwLock<Host>>>) {
        for host in hosts.iter() {
            self.all_hosts.insert(host.read().unwrap().name.clone(), Arc::clone(&host));
            self.remaining_hosts.insert(host.read().unwrap().name.clone(), Arc::clone(&host));
        }
    }

    pub fn fail_host(&mut self, host: &Arc<RwLock<Host>>) {
        // FIXME - we should really keep all_hosts seperate from unfailed_hosts
        self.remaining_hosts.remove(&host.read().unwrap().name);
        self.failed_hosts.insert(host.read().unwrap().name, Arc::clone(&host));
    }


    // =================================================================================
    // SIGNPOSTS

    // FIXME: we need a method set_hosts that clears all_hosts

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        self.playbook_path = Some(path_as_string(&path));
        self.playbook_directory = Some(directory_as_string(&path));
    }

    pub fn set_task(&mut self, task_name: String) {
        self.task = Some(task_name.clone());
    }

    pub fn set_play(&mut self, play: &Play) {
        self.play = Some(play.name.clone());
    }

    pub fn get_play_name(&self) -> String {
        return match &self.play {
            Some(x) => x.clone(),
            None => panic!("attempting to read a play name before plays have been evaluated")
        }
    }

    pub fn set_role(&mut self, role_name: String, role_path: String) {
        self.role_name  = Some(role_name.clone());
        self.role_path = Some(role_path.clone());
    }

    pub fn unset_role(&mut self) {
        self.role_name = None;
        self.role_path = None;
    }
    

    /* FIXME: doesn't make sense, I think?
    pub fn set_remote_user(&mut self, play: &Play, default_username: String) {
        // FIXME: get current logged in username or update docs
        match &play.ssh_user {
            Some(x) => { self.remote_user = Some(x.clone()) },
            None => { self.remote_user = Some(String::from("root")); }
        }
    }
    */

    pub fn get_remote_user(&self, host: &Arc<RwLock<Host>>) -> String {
        // FIXME: default only if host doesn't have an answer in blended variables
        // FIXME: we can also see if there is a value set on the play
        return match &self.remote_user {
            Some(x) => x.clone(),
            None => String::from("root")
        }
    }

    pub fn get_remote_port(&self, host: &Arc<RwLock<Host>>) -> usize {
        // FIXME: default only if host doesn't have an answer in blended variables
        // FIXME: we can also see if there is a value set on the play
        return 22usize;

    }

    */

    // ==================================================================================
    // STATISTICS

    pub fn get_role_count(&self) -> usize {
        return self.role_count;
    }

    pub fn increment_role_count(&mut self) {
        self.role_count = self.role_count + 1;
    }

    pub fn get_task_count(&self) -> usize {
        return self.task_count;
    }

    pub fn increment_task_count(&mut self) {
        self.task_count = self.task_count + 1;
    }


}