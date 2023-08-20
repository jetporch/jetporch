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
use crate::tasks::common::TaskProperty;
use std::path::PathBuf;
use std::collections::{HashMap};
use crate::inventory::hosts::Host;
use std::sync::{Arc,RwLock};
use crate::registry::list::Task;

pub struct PlaybookContext {

    // FIXME: needs a .reset() method but keeps failed_hosts and refuses to add hosts to 
    // the targetted list if they had failures
    
    pub playbook_path: Option<String>,
    pub playbook_directory: Option<String>,
    pub play: Option<String>,
    
    pub role: Option<String>,
    pub role_path: Option<String>,
    pub role_name: Option<String>,
    pub role_count: usize,

    pub task_count: usize,
    pub task: Option<String>,
    
    pub seen_hosts: HashMap<String, Arc<RwLock<Host>>>,
    pub targetted_hosts: HashMap<String, Arc<RwLock<Host>>>,
    pub failed_hosts: HashMap<String, Arc<RwLock<Host>>>,

    pub attempted_count_for_host: HashMap<String, usize>,
    pub adjusted_count_for_host:  HashMap<String, usize>,
    pub created_count_for_host:   HashMap<String, usize>,
    pub removed_count_for_host:   HashMap<String, usize>,
    pub modified_count_for_host:  HashMap<String, usize>,
    pub executed_count_for_host:  HashMap<String, usize>,
    pub failed_count_for_host:    HashMap<String, usize>,
    // FIXME: should this be here, it's a property of the connection? is it used?
    pub ssh_remote_user: Option<String>,

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
            seen_hosts: HashMap::new(),
            targetted_hosts: HashMap::new(),
            failed_hosts: HashMap::new(),
            role_path: None,
            role_name: None,
            // FIXME: look through cli commands - this is part of the runstate, not the context
            // and should be modified.  Remove the methods below that access it / port
            ssh_remote_user: None, 
            adjusted_count_for_host:  HashMap::new(),
            attempted_count_for_host: HashMap::new(),
            created_count_for_host:   HashMap::new(),
            removed_count_for_host:   HashMap::new(),
            modified_count_for_host:  HashMap::new(),
            executed_count_for_host:  HashMap::new(),
            failed_count_for_host:    HashMap::new(),
        }
    }

    // ===============================================================================
    // HOST TARGETTING

    pub fn get_remaining_hosts(&self) -> HashMap<String, Arc<RwLock<Host>>> {
        let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
        for (k,v) in self.targetted_hosts.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
        return results;
    }

    pub fn set_targetted_hosts(&mut self, hosts: &Vec<Arc<RwLock<Host>>>) {
        self.targetted_hosts.clear();
        for host in hosts.iter() {
            let hostname = host.read().unwrap().name.clone();
            match self.failed_hosts.contains_key(&hostname) {
                true => {},
                false => { 
                    self.seen_hosts.insert(hostname.clone(), Arc::clone(&host));
                    self.targetted_hosts.insert(hostname.clone(), Arc::clone(&host)); 
                }
            }
        }
    }

    pub fn fail_host(&mut self, host: &Arc<RwLock<Host>>) {
        // FIXME - we should really keep all_hosts seperate from unfailed_hosts
        let host2 = host.read().unwrap();
        let hostname = host2.name.clone();
        self.targetted_hosts.remove(&hostname);
        self.failed_hosts.insert(hostname.clone(), Arc::clone(&host));
    }


    // =================================================================================
    // SIGNPOSTS

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        self.playbook_path = Some(path_as_string(&path));
        self.playbook_directory = Some(directory_as_string(&path));
    }

    pub fn set_task(&mut self, task: &Task) {
        self.task = Some(task.get_property(TaskProperty::Name)); 
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
            Some(x) => { self.ssh_remote_user = Some(x.clone()) },
            None => { self.ssh_remote_user = Some(String::from("root")); }
        }
    }
    */


    // FIXME: move to runstate???

    pub fn get_ssh_remote_user(&self, host: &Arc<RwLock<Host>>) -> String {
        // FIXME: default only if host doesn't have an answer in blended variables
        // FIXME: we can also see if there is a value set on the play
        return match &self.ssh_remote_user {
            Some(x) => x.clone(),
            None => String::from("root")
        }
    }

    // FIXME: move to runstate???

    pub fn get_ssh_remote_port(&self, host: &Arc<RwLock<Host>>) -> usize {
        // FIXME: default only if host doesn't have an answer in blended variables
        // FIXME: we can also see if there is a value set on the play
        return 22usize;
    }

    // ==================================================================================
    // STATISTICS

    // FIXME: might want to track task count differently from handler count?

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

    pub fn increment_attempted_for_host(&mut self, host: &String) {
        *self.attempted_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }
    pub fn increment_created_for_host(&mut self, host: &String) {
        *self.created_count_for_host.entry(host.clone()).or_insert(0) += 1;
        *self.adjusted_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }
    pub fn increment_removed_for_host(&mut self, host: &String) {
        *self.removed_count_for_host.entry(host.clone()).or_insert(0) += 1;
        *self.adjusted_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }
    pub fn increment_modified_for_host(&mut self, host: &String) {
        *self.modified_count_for_host.entry(host.clone()).or_insert(0) += 1;
        *self.adjusted_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }
    pub fn increment_executed_for_host(&mut self, host: &String) {
        *self.executed_count_for_host.entry(host.clone()).or_insert(0) += 1;
        *self.adjusted_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }
    pub fn increment_failed_for_host(&mut self, host: &String) {
        *self.failed_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }

    pub fn get_total_attempted_count(&self) -> usize {
        return self.attempted_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_creation_count(&self) -> usize {
        return self.created_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_modified_count(&self) -> usize{
        return self.modified_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_removal_count(&self) -> usize{
        return self.removed_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_executions_count(&self) -> usize {
        return self.executed_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_failed_count(&self) -> usize{
        return self.failed_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }
    pub fn get_total_adjusted_count(&self) -> usize {
        return self.adjusted_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }

    pub fn get_hosts_attempted_count(&self) -> usize {
        return self.attempted_count_for_host.keys().len();
    }
    pub fn get_hosts_creation_count(&self) -> usize {
        return self.created_count_for_host.keys().len();
    }
    pub fn get_hosts_modified_count(&self) -> usize {
        return self.modified_count_for_host.keys().len();
    }
    pub fn get_hosts_removal_count(&self) -> usize {
        return self.removed_count_for_host.keys().len();
    }
    pub fn get_hosts_executions_count(&self) -> usize {
        return self.executed_count_for_host.keys().len();
    }
    pub fn get_hosts_failed_count(&self) -> usize {
        return self.failed_count_for_host.keys().len();
    }
    pub fn get_hosts_adjusted_count(&self) -> usize {
        return self.adjusted_count_for_host.keys().len();
    }
    pub fn get_hosts_seen_count(&self) -> usize {
        return self.seen_hosts.keys().len();
    }
    

}