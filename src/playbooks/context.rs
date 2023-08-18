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
use std::collections::HashSet;
use std::sync::{RwLock};

pub struct PlaybookContext {
    pub playbook_path: Option<String>,
    pub playbook_directory: Option<String>,
    pub play: Option<String>,
    pub role: Option<String>,
    pub role_count: usize,
    pub task_count: usize,
    pub task: Option<String>,
    pub host: Option<String>,
    pub all_hosts: HashSet<String>,
    pub role_path: Option<String>,
    pub role_name: Option<String>,
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
            all_hosts: HashSet::new(),
            role_path: None,
            role_name: None,
            remote_user: None
        }
    }

    // get all selected hosts in the play
    // FIXME: need a method for non-failed hosts
    pub fn get_all_hosts(&self) -> HashSet<String> {
        let mut results : HashSet<String> = HashSet::new();
        for host in self.all_hosts.iter() {
            results.insert(host.clone());
        }
        return results;
    }

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
    
    pub fn set_remote_user(&mut self, play: &Play, default_username: String) {
        // FIXME: get current logged in username or update docs
        match &play.ssh_user {
            Some(x) => { self.remote_user = Some(x.clone()) },
            None => { self.remote_user = Some(String::from("root")); }
        }
    }

    pub fn get_remote_user(&self, host: &String) -> String {
        return match &self.remote_user {
            Some(x) => x.clone(),
            None => String::from("root")
        }
    }

    pub fn get_remote_port(&self, host: &String) -> usize {
        // notice this doesn't really use the context.
        // FIXME: check the variables + host variables
        return 22usize;

    }

    pub fn fail_host(&mut self, host: &String){
        // FIXME - we should really keep all_hosts seperate from unfailed_hosts
        panic!("fail_host is not implemented yet");
    }

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