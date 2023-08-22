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

use crate::util::io::{path_as_string,directory_as_string};
use crate::playbooks::language::Play;
use std::path::PathBuf;
use std::collections::{HashMap};
use crate::inventory::hosts::Host;
use std::sync::{Arc,RwLock};
use crate::connection::cache::ConnectionCache;
use crate::registry::list::Task;
use crate::util::yaml::blend_variables;
use crate::playbooks::templar::Templar;

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
    
    seen_hosts:               HashMap<String, Arc<RwLock<Host>>>,
    targetted_hosts:          HashMap<String, Arc<RwLock<Host>>>,
    failed_hosts:             HashMap<String, Arc<RwLock<Host>>>,

    attempted_count_for_host: HashMap<String, usize>,
    adjusted_count_for_host:  HashMap<String, usize>,
    created_count_for_host:   HashMap<String, usize>,
    removed_count_for_host:   HashMap<String, usize>,
    modified_count_for_host:  HashMap<String, usize>,
    executed_count_for_host:  HashMap<String, usize>,
    passive_count_for_host:   HashMap<String, usize>,
    failed_count_for_host:    HashMap<String, usize>,
    
    pub default_remote_user:  Option<String>,
    pub connection_cache:     RwLock<ConnectionCache>,
    pub templar:              RwLock<Templar>,

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
            default_remote_user: None, 
            adjusted_count_for_host:  HashMap::new(),
            attempted_count_for_host: HashMap::new(),
            created_count_for_host:   HashMap::new(),
            removed_count_for_host:   HashMap::new(),
            modified_count_for_host:  HashMap::new(),
            executed_count_for_host:  HashMap::new(),
            passive_count_for_host:   HashMap::new(),
            failed_count_for_host:    HashMap::new(),
            connection_cache:         RwLock::new(ConnectionCache::new()),
            templar:                  RwLock::new(Templar::new())
            
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
        self.task = Some(task.get_display_name());
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
    
    pub fn set_default_remote_user(&mut self, default_username: Option<String>) {
        match default_username {
            Some(x) => { self.default_remote_user = Some(x) },
            None => { }
        }
    }

    pub fn get_default_remote_user(&self, _host: &Arc<RwLock<Host>>) -> Option<String> {
        return self.default_remote_user.clone();
    }

    // ==================================================================================
    // VARIABLES

    pub fn get_complete_blended_variables(&self, host: &Arc<RwLock<Host>>) -> String  {
        // !!!
        // !!!
        // FIXME: load in defaults then inventory then vars/vars_files - keep role vars seperate
        // !!!
        // !!!
        let blended = String::from("");
        let host_blended = host.read().unwrap().get_blended_variables();
        //let context_blended = context.read().get_blended_variables();
        let blended2 = blend_variables(&host_blended, &blended);
        //blended = blend_variables(&host_blended, &context_blended);
        return blended2
    }

    pub fn get_complete_blended_variables_mapping(&self, host: &Arc<RwLock<Host>>) -> HashMap<String, serde_yaml::Value> {
        let mut complete_blended = self.get_complete_blended_variables(host);
        if complete_blended.eq("null\n") {
            complete_blended = String::from("unset: true");
        }
        let mut vars: HashMap<String,serde_yaml::Value> = serde_yaml::from_str(&complete_blended).unwrap();
        return vars;
    }

    pub fn render_template(&self, template: &String, host: &Arc<RwLock<Host>>) -> Result<String,String> {
        let vars = self.get_complete_blended_variables_mapping(host);
        return self.templar.read().unwrap().render(template, vars);
    }

    pub fn get_ssh_connection_details(&self, host: &Arc<RwLock<Host>>) -> (String,String,i64) {
        let vars = self.get_complete_blended_variables_mapping(host);
        let host2 = host.read().unwrap();

        let remote_hostname = match vars.contains_key(&String::from("jet_ssh_remote_hostname")) {
            true => match vars.get(&String::from("jet_ssh_remote_hostname")).unwrap().as_str() {
                Some(x) => String::from(x),
                None => host2.name.clone()
            },
            false => host2.name.clone()
        };
        let remote_user = match vars.contains_key(&String::from("jet_ssh_remote_user")) {
            true => match vars.get(&String::from("jet_ssh_remote_user")).unwrap().as_str() {
                Some(x) => String::from(x),
                None => String::from("root")
            },
            false => match &self.default_remote_user {
                Some(x) => x.clone(),
                None => String::from("root")
            }
        };
        let remote_port = match vars.contains_key(&String::from("jet_ssh_remote_port")) {
            true => match vars.get(&String::from("jet_ssh_remote_port")).unwrap().as_i64() {
                Some(x) => x,
                None => 22
            },
            false => 22
        };

        return (remote_hostname, remote_user, remote_port)
    } 

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

    pub fn increment_passive_for_host(&mut self, host: &String) {
        *self.passive_count_for_host.entry(host.clone()).or_insert(0) += 1;
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

    pub fn get_total_passive_count(&self) -> usize {
        return self.passive_count_for_host.values().fold(0, |ttl, &x| ttl + x);
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

    pub fn get_hosts_passive_count(&self) -> usize {
        return self.passive_count_for_host.keys().len();
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