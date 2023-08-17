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
    pub playbook_path: RwLock<Option<String>>,
    pub playbook_directory: RwLock<Option<String>>,
    pub play: RwLock<Option<String>>,
    pub role: RwLock<Option<String>>,
    pub task: RwLock<Option<String>>,
    pub host: RwLock<Option<String>>,
    pub all_hosts: RwLock<HashSet<String>>,
    pub role_path: RwLock<Option<String>>,
    pub role_name: RwLock<Option<String>>,
    pub remote_user: RwLock<Option<String>>,
}

impl PlaybookContext {

    pub fn new() -> Self {
        Self {
            playbook_path: RwLock::new(None),
            playbook_directory: RwLock::new(None),
            play: RwLock::new(None),
            role: RwLock::new(None),
            task: RwLock::new(None),
            host: RwLock::new(None),
            all_hosts: RwLock::new(HashSet::new()),
            role_path: RwLock::new(None),
            role_name: RwLock::new(None),
            remote_user: RwLock::new(None)
        }
    }

    // get all selected hosts in the play
    // FIXME: need a method for non-failed hosts
    pub fn get_all_hosts(&self) -> HashSet<String> {
        let hosts = self.all_hosts.read().unwrap();
        let mut results : HashSet<String> = HashSet::new();
        for host in hosts.iter() {
            results.insert(host.clone());
        }
        return results;
    }

    // FIXME: we need a method set_hosts that clears all_hosts

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        *self.playbook_path.write().unwrap() = Some(path_as_string(&path));
        *self.playbook_directory.write().unwrap() = Some(directory_as_string(&path));
    }

    pub fn set_task(&mut self, task_name: String) {
        *self.task.write().unwrap() = Some(task_name.clone());
    }

    pub fn set_play(&mut self, play: &Play) {
        *self.play.write().unwrap() = Some(play.name.clone());
    }

    pub fn set_role(&mut self, role_name: String, role_path: String) {
        *self.role_name.write().unwrap() = Some(role_name.clone());
        *self.role_path.write().unwrap() = Some(role_path.clone());
    }

    pub fn unset_role(&mut self) {
        *self.role_name.write().unwrap() = None;
        *self.role_path.write().unwrap() = None;
    }
    
    pub fn set_remote_user(&mut self, play: &Play, default_username: String) {
        // FIXME: get current logged in username or update docs
        match &play.ssh_user {
            Some(x) => { *self.remote_user.write().unwrap() = Some(x.clone()) },
            None => { *self.remote_user.write().unwrap() = Some(String::from("root")); }
        }
    }

    pub fn get_remote_user(&mut self, host: String) -> String {
        let default = self.remote_user.read().unwrap();
        if default.is_some() {
            let x = default.as_ref().unwrap();
            return x.clone();
        } else {
            return String::from("root"); 
        }
    }

    pub fn get_remote_port(&mut self, host: String) -> usize {
        // notice this doesn't really use the context.
        // FIXME: check the variables + host variables
        return 22usize;

    }

    pub fn fail_host(&mut self, host: String){
        // FIXME - we should really keep all_hosts seperate from unfailed_hosts
        panic!("fail_host is not implemented yet");
    }


}