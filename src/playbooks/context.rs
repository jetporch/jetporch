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
use crate::playbooks::language::{Play,Role,RoleInvocation};
use std::path::PathBuf;
use std::collections::HashMap;
use crate::inventory::hosts::Host;
use std::sync::{Arc,RwLock};
use crate::connection::cache::ConnectionCache;
use crate::registry::list::Task;
use crate::util::yaml::blend_variables;
use crate::playbooks::templar::{Templar,TemplateMode};
use crate::cli::parser::CliParser;
use crate::handle::template::BlendTarget;
use std::ops::Deref;
use std::env;
use guid_create::GUID;

// the playbook traversal state, and a little bit more than that.
// the playbook context keeps track of where we are in a playbook
// execution and various results/stats along the way.

pub struct PlaybookContext {

    pub verbosity: u32,

    pub playbook_path: Option<String>,
    pub playbook_directory: Option<String>,
    pub play: Option<String>,
    
    pub role: Option<Role>,
    pub role_path: Option<String>,
    pub play_count: usize,
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
    matched_count_for_host:   HashMap<String, usize>,
    skipped_count_for_host:   HashMap<String, usize>,
    failed_count_for_host:    HashMap<String, usize>,
    
    // TODO: some of these don't need to be pub.
    pub failed_tasks:           usize,
    pub defaults_storage:       RwLock<serde_yaml::Mapping>,
    pub vars_storage:           RwLock<serde_yaml::Mapping>,
    pub role_defaults_storage:  RwLock<serde_yaml::Mapping>,
    pub role_vars_storage:      RwLock<serde_yaml::Mapping>,
    pub env_storage:            RwLock<serde_yaml::Mapping>,
    
    pub connection_cache:     RwLock<ConnectionCache>,
    pub templar:              RwLock<Templar>,

    pub ssh_user:             String,
    pub ssh_port:             i64,
    pub sudo:                 Option<String>,
    extra_vars:               serde_yaml::Value,

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
            play_count : 0,
            role_count : 0,
            task_count : 0,
            seen_hosts: HashMap::new(),
            targetted_hosts: HashMap::new(),
            failed_hosts: HashMap::new(),
            role_path: None,
            adjusted_count_for_host:  HashMap::new(),
            attempted_count_for_host: HashMap::new(),
            created_count_for_host:   HashMap::new(),
            removed_count_for_host:   HashMap::new(),
            modified_count_for_host:  HashMap::new(),
            executed_count_for_host:  HashMap::new(),
            passive_count_for_host:   HashMap::new(),
            matched_count_for_host:   HashMap::new(),
            failed_count_for_host:    HashMap::new(),
            skipped_count_for_host:   HashMap::new(),
            connection_cache:         RwLock::new(ConnectionCache::new()),
            templar:                  RwLock::new(Templar::new()),
            defaults_storage:         RwLock::new(serde_yaml::Mapping::new()),
            vars_storage:             RwLock::new(serde_yaml::Mapping::new()),
            role_vars_storage:        RwLock::new(serde_yaml::Mapping::new()),
            role_defaults_storage:    RwLock::new(serde_yaml::Mapping::new()),
            env_storage:              RwLock::new(serde_yaml::Mapping::new()),
            ssh_user:                 parser.default_user.clone(),
            ssh_port:                 parser.default_port,
            sudo:                     parser.sudo.clone(),
            extra_vars:               parser.extra_vars.clone(),
        };
        s.load_environment();
        return s;
    }

    // the remaining hosts in a play are those that have not failed yet
    // other functions remove these hosts from the list.

    pub fn get_remaining_hosts(&self) -> HashMap<String, Arc<RwLock<Host>>> {
        let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
        for (k,v) in self.targetted_hosts.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
        return results;
    }

    // SSH details are set in traversal and may come from the playbook or
    // or CLI options. These values are not guaranteed to be used as magic
    // variables could still exist in inventory for particular hosts

    pub fn set_ssh_user(&mut self, ssh_user: &String) {
        self.ssh_user = ssh_user.clone();
    }

    pub fn set_ssh_port(&mut self, ssh_port: i64) {
        self.ssh_port = ssh_port;
    }

    // used in traversal to tell the context what the current set of possible
    // hosts is.

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

    // called when a host returns an unacceptable final response.  removes
    // the host from the targetted pool for the play.  when no hosts
    // remain the entire play will fail.

    pub fn fail_host(&mut self, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        let hostname = host2.name.clone();
        self.failed_tasks = self.failed_tasks + 1;

        
        self.targetted_hosts.remove(&hostname);
        self.failed_hosts.insert(hostname.clone(), Arc::clone(&host));
    }

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        self.playbook_path = Some(path_as_string(&path));
        self.playbook_directory = Some(directory_as_string(&path));
    }

    pub fn set_task(&mut self, task: &Task) {
        self.task = Some(task.get_display_name());
    }

    pub fn set_play(&mut self, play: &Play) {
        self.play = Some(play.name.clone());
        self.play_count = self.play_count + 1;
    }

    pub fn get_play_name(&self) -> String {
        return match &self.play {
            Some(x) => x.clone(),
            None => panic!("attempting to read a play name before plays have been evaluated")
        }
    }

    pub fn set_role(&mut self, role: &Role, invocation: &RoleInvocation, role_path: &String) {
        self.role = Some(role.clone());
        self.role_path = Some(role_path.clone());
        if role.defaults.is_some() { 
             *self.role_defaults_storage.write().unwrap() = role.defaults.as_ref().unwrap().clone();
        }
        if invocation.vars.is_some() { 
            *self.role_vars_storage.write().unwrap() = invocation.vars.as_ref().unwrap().clone();
        }
    }

    pub fn unset_role(&mut self) {
        self.role = None;
        self.role_path = None;
        self.role_defaults_storage.write().unwrap().clear();
        self.role_vars_storage.write().unwrap().clear();
    }

    // template functions need to access all the variables about a host taking variable precendence rules into effect
    // to get a dictionary of variables to use in template expressions

    pub fn get_complete_blended_variables(&self, host: &Arc<RwLock<Host>>, blend_target: BlendTarget) -> serde_yaml::Mapping  {
        let blended = self.get_complete_blended_variables_as_value(host, blend_target);
        return match blended {
            serde_yaml::Value::Mapping(x) => x,
            _ => panic!("unexpected, get_blended_variables produced a non-mapping (3)")
        };
    }

    pub fn get_complete_blended_variables_as_value(&self, host: &Arc<RwLock<Host>>, blend_target: BlendTarget) -> serde_yaml::Value  {
        
        let mut blended = serde_yaml::Value::from(serde_yaml::Mapping::new());
        let src1 = self.defaults_storage.read().unwrap();
        let src1a = src1.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src1a.clone()));
        
        let src1r = self.role_defaults_storage.read().unwrap();
        let src1ar = src1r.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src1ar.clone()));

        let src2 = host.read().unwrap().get_blended_variables();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src2));

        let src3 = self.vars_storage.read().unwrap();
        let src3a = src3.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src3a.clone()));

        let src3r = self.role_vars_storage.read().unwrap();
        let src3ar = src3r.deref();
        blend_variables(&mut blended, serde_yaml::Value::Mapping(src3ar.clone()));

        blend_variables(&mut blended, self.extra_vars.clone());

        match blend_target {
            BlendTarget::NotTemplateModule => { },
            BlendTarget::TemplateModule => {
                // for security reasons env vars from security tools like 'op run' are only exposed to the template module
                // to prevent accidental leakage into logs and history
                let src4 = self.env_storage.read().unwrap();
                let src4a = src4.deref();
                blend_variables(&mut blended, serde_yaml::Value::Mapping(src4a.clone()));
            }
        };
        return blended;
    }

    // template code is not used here directly, but in handle/template.rs, which passes back through here, since
    // only the context knows all the variables from the playbook traversal to fill in and how to blend
    // variables in the correct order.

    pub fn render_template(&self, template: &String, host: &Arc<RwLock<Host>>, blend_target: BlendTarget, template_mode: TemplateMode) -> Result<String,String> {
        let vars = self.get_complete_blended_variables(host, blend_target);
        return self.templar.read().unwrap().render(template, vars, template_mode);
    }

    // testing conditions for truthiness works much like templating strings

    pub fn test_condition(&self, expr: &String, host: &Arc<RwLock<Host>>, tm: TemplateMode) -> Result<bool,String> {
        let vars = self.get_complete_blended_variables(host, BlendTarget::NotTemplateModule);
        return self.templar.read().unwrap().test_condition(expr, vars, tm);
    }

    // a version of template evaluation that allows some additional variables, for example from a module

    pub fn test_condition_with_extra_data(&self, expr: &String, host: &Arc<RwLock<Host>>, vars_input: serde_yaml::Mapping, tm: TemplateMode) -> Result<bool,String> {
        let mut vars = self.get_complete_blended_variables_as_value(host, BlendTarget::NotTemplateModule);
        blend_variables(&mut vars, serde_yaml::Value::Mapping(vars_input));
        return match vars {
            serde_yaml::Value::Mapping(x) => self.templar.read().unwrap().test_condition(expr, x, tm),
            _ => { panic!("impossible input to test_condition"); }
        };
    }

    // when a host needs to connect over SSH it asks this function - we can use some settings configured
    // already on the context or check some variables in inventory.

    // FIXME: this should return a struct

    pub fn get_ssh_connection_details(&self, host: &Arc<RwLock<Host>>) -> (String,String,i64,Option<String>,Option<String>) {

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
                Some(x) => {
                    x
                },
                None => {
                    self.ssh_port
                }
            },
            false => {
                self.ssh_port
            }
        };
        let keyfile : Option<String> = match vars.contains_key(&String::from("jet_ssh_private_key_file")) {
            true => match vars.get(&String::from("jet_ssh_private_key_file")).unwrap().as_str() {
                Some(x) => Some(String::from(x)),
                None => None
            },
            false => None
        };
        let passphrase : Option<String> = match vars.contains_key(&String::from("jet_ssh_private_key_passphrase")) {
            true => match vars.get(&String::from("jet_ssh_private_key_passphrase")).unwrap().as_str() {
                Some(x) => Some(String::from(x)),
                None =>  None
            },
            false => match env::var("JET_SSH_PRIVATE_KEY_PASSPHRASE") {
                Ok(x) => Some(x),
                Err(_) => None
            }
        };


        return (remote_hostname, remote_user, remote_port, keyfile, passphrase)
    } 

    // loads environment variables into the context, adding an "ENV_foo" prefix
    // to each environment variable "foo". These variables will only be made available
    // to the template module since we use them for secret management features.

    pub fn load_environment(&mut self) {
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
            }
        }
    }

    // various functions in Jet make use of GUIDs, for example for temp file locations

    pub fn get_guid(&self) -> String {
        return GUID::rand().to_string();
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

    pub fn increment_matched_for_host(&mut self, host: &String) {
        *self.matched_count_for_host.entry(host.clone()).or_insert(0) += 1;
    }

    pub fn increment_skipped_for_host(&mut self, host: &String) {
        *self.skipped_count_for_host.entry(host.clone()).or_insert(0) += 1;
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

    pub fn get_total_matched_count(&self) -> usize {
        return self.matched_count_for_host.values().fold(0, |ttl, &x| ttl + x);
    }

    pub fn get_total_skipped_count(&self) -> usize {
        return self.skipped_count_for_host.values().fold(0, |ttl, &x| ttl + x);
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

    pub fn get_hosts_matched_count(&self) -> usize {
        return self.matched_count_for_host.keys().len();
    }

    pub fn get_hosts_skipped_count(&self) -> usize {
        return self.skipped_count_for_host.keys().len();
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