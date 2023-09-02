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
use std::collections::HashMap;
use crate::inventory::hosts::Host;
use std::sync::{Arc,RwLock};
use crate::connection::cache::ConnectionCache;
use crate::registry::list::Task;
use crate::util::yaml::blend_variables;
use crate::playbooks::templar::Templar;
use crate::cli::parser::CliParser;
use crate::handle::template::BlendTarget;
use std::ops::Deref;
use std::env;

// the playbook context keeps track of where we are in a playbook
// execution and various results/stats along the way

pub struct PlaybookContext {

    pub verbosity: u32,

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
    
    pub failed_tasks:           usize,
    pub defaults_storage:       RwLock<serde_yaml::Mapping>,
    pub vars_storage:           RwLock<serde_yaml::Mapping>,
    pub role_defaults_storage:  RwLock<serde_yaml::Mapping>,
    pub env_storage:            RwLock<serde_yaml::Mapping>,
    
    //pub default_remote_user:  String,
    pub connection_cache:     RwLock<ConnectionCache>,
    pub templar:              RwLock<Templar>,

    pub ssh_user:             String,
    pub ssh_port:             i64

}

impl PlaybookContext {

    pub fn new(parser: &CliParser) -> Self {
        let mut s = Self {
            verbosity: parser.verbosity,
            playbook_path: None,
            playbook_directory: None,
            failed_tasks: 0,
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
            adjusted_count_for_host:  HashMap::new(),
            attempted_count_for_host: HashMap::new(),
            created_count_for_host:   HashMap::new(),
            removed_count_for_host:   HashMap::new(),
            modified_count_for_host:  HashMap::new(),
            executed_count_for_host:  HashMap::new(),
            passive_count_for_host:   HashMap::new(),
            failed_count_for_host:    HashMap::new(),
            connection_cache:         RwLock::new(ConnectionCache::new()),
            templar:                  RwLock::new(Templar::new()),
            defaults_storage:         RwLock::new(serde_yaml::Mapping::new()),
            vars_storage:             RwLock::new(serde_yaml::Mapping::new()),
            role_defaults_storage:    RwLock::new(serde_yaml::Mapping::new()),
            env_storage:              RwLock::new(serde_yaml::Mapping::new()),
            ssh_user:                 parser.default_user.clone(),
            ssh_port:                 22
        };
        s.load_environment();
        return s;
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

    pub fn set_ssh_user(&mut self, ssh_user: &String) {
        self.ssh_user = ssh_user.clone();
    }

    pub fn set_ssh_port(&mut self, ssh_port: i64) {
        self.ssh_port = ssh_port;
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
        self.failed_tasks = self.failed_tasks + 1;

        
        self.targetted_hosts.remove(&hostname);
        self.failed_hosts.insert(hostname.clone(), Arc::clone(&host));
    }

    pub fn syntax_fail_host(&mut self, host: &Arc<RwLock<Host>>) {
        self.failed_tasks = self.failed_tasks + 1;
        let host2 = host.read().unwrap();
        let hostname = host2.name.clone();
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

    // ==================================================================================
    // VARIABLES

    pub fn get_complete_blended_variables(&self, host: &Arc<RwLock<Host>>, blend_target: BlendTarget) -> serde_yaml::Mapping  {
        let mut blended = serde_yaml::Value::from(serde_yaml::Mapping::new());
        let src1 = self.defaults_storage.read().unwrap();
        //.deref();
        let src1a = src1.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src1a.clone()));
        let src2 = host.read().unwrap().get_blended_variables();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src2));
        let src3 = self.vars_storage.read().unwrap();
        let src3a = src3.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src3a.clone()));

        match blend_target {
            BlendTarget::NotTemplateModule => {},
            BlendTarget::TemplateModule => {
                // for security reasons env vars from security tools like 'op run' are only exposed to the template module
                // to prevent accidental leakage into logs and history
                let src4 = self.env_storage.read().unwrap();
                let src4a = src4.deref();
                blend_variables(&mut blended, serde_yaml::Value::Mapping(src4a.clone()));
            }
        };
        return match blended {
            serde_yaml::Value::Mapping(x) => x,
            _ => panic!("unexpected, get_blended_variables produced a non-mapping (3)")
        };
    }

    pub fn render_template(&self, template: &String, host: &Arc<RwLock<Host>>, blend_target: BlendTarget) -> Result<String,String> {
        let vars = self.get_complete_blended_variables(host, blend_target);
        return self.templar.read().unwrap().render(template, vars);
    }

    pub fn test_cond(&self, expr: &String, host: &Arc<RwLock<Host>>) -> Result<bool,String> {
        let vars = self.get_complete_blended_variables(host, BlendTarget::NotTemplateModule);
        return self.templar.read().unwrap().test_cond(expr, vars);
    }

    pub fn get_ssh_connection_details(&self, host: &Arc<RwLock<Host>>) -> (String,String,i64) {
        let vars = self.get_complete_blended_variables(host,BlendTarget::NotTemplateModule);
        let host2 = host.read().unwrap();

        let remote_hostname = match vars.contains_key(&String::from("jet_ssh_hostname")) {
            true => match vars.get(&String::from("jet_ssh_hostname")).unwrap().as_str() {
                Some(x) => String::from(x),
                None => host2.name.clone()
            },
            false => host2.name.clone()
        };
        let remote_user = match vars.contains_key(&String::from("jet_ssh_user")) {
            true => match vars.get(&String::from("jet_ssh_user")).unwrap().as_str() {
                Some(x) => String::from(x),
                None => self.ssh_user.clone()
            },
            false => self.ssh_user.clone()
        };
        let remote_port = match vars.contains_key(&String::from("jet_ssh_port")) {
            true => match vars.get(&String::from("jet_ssh_port")).unwrap().as_i64() {
                Some(x) => x,
                None => self.ssh_port
            },
            false => self.ssh_port
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
    
    pub fn load_environment(&mut self) -> () {
        let mut my_env = self.env_storage.write().unwrap();
        // some common environment variables that may occur are not useful for playbooks
        // or they have no need to share that with other hosts
        let do_not_load = vec![
            "OLDPWD",
            "PWD",
            "SHLVL",
            "SSH_AUTH_SOCK",
            "SSH_AGENT_PID",
            "TERM_SESSION_ID",
            "XPC_FLAGS",
            "XPC_SERVICE_NAME",
            "_"
        ];
        
        for (k,v) in env::vars() {
            if ! do_not_load.contains(&k.as_str()) {
                my_env.insert(serde_yaml::Value::String(format!("ENV_{k}")) , serde_yaml::Value::String(v));
            } else {
            }
        }
    }

}