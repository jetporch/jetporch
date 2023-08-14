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

use std::sync::Mutex;
use std::sync::Arc;
use crate::util::io::{path_as_string,directory_as_string};
use crate::playbooks::language::Play;
use std::path::PathBuf;
use std::collections::HashSet;

pub struct PlaybookContext {
    pub playbook_path: Arc<Mutex<Option<String>>>,
    pub playbook_directory: Arc<Mutex<Option<String>>>,
    pub play: Arc<Mutex<Option<String>>>,
    pub role: Arc<Mutex<Option<String>>>,
    pub task: Arc<Mutex<Option<String>>>,
    pub host: Arc<Mutex<Option<String>>>,
    pub all_hosts: Arc<Mutex<HashSet<String>>>,
    pub role_path: Arc<Mutex<Option<String>>>,
    pub role_name: Arc<Mutex<Option<String>>>,
    pub remote_user: Arc<Mutex<Option<String>>>
}

impl PlaybookContext {

    pub fn new() -> Self {
        Self {
            playbook_path: Arc::new(Mutex::new(None)),
            playbook_directory: Arc::new(Mutex::new(None)),
            play: Arc::new(Mutex::new(None)),
            role: Arc::new(Mutex::new(None)),
            task: Arc::new(Mutex::new(None)),
            host: Arc::new(Mutex::new(None)),
            all_hosts: Arc::new(Mutex::new(HashSet::new())),
            role_path: Arc::new(Mutex::new(None)),
            role_name: Arc::new(Mutex::new(None)),
            remote_user: Arc::new(Mutex::new(None))
        }
    }

    // get all selected hosts in the play
    // FIXME: need a method for non-failed hosts
    pub fn get_all_hosts(&self) -> HashSet<String> {
        let hosts = self.all_hosts.lock().unwrap();
        let mut results : HashSet<String> = HashSet::new();
        for host in hosts.iter() {
            results.insert(host.clone());
        }
        return results;
    }

    // FIXME: we need a method set_hosts that clears all_hosts

    pub fn set_playbook_path(&mut self, path: &PathBuf) {
        *self.playbook_path.lock().unwrap() = Some(path_as_string(&path));
        *self.playbook_directory.lock().unwrap() = Some(directory_as_string(&path));
    }

    pub fn set_task(&mut self, task_name: String) {
        *self.task.lock().unwrap() = Some(task_name.clone());
    }

    pub fn set_play(&mut self, play: &Play) {
        *self.play.lock().unwrap() = Some(play.name.clone());
    }

    pub fn set_role(&mut self, role_name: String, role_path: String) {
        *self.role_name.lock().unwrap() = Some(role_name.clone());
        *self.role_path.lock().unwrap() = Some(role_path.clone());
    }

    pub fn unset_role(&mut self) {
        *self.role_name.lock().unwrap() = None;
        *self.role_path.lock().unwrap() = None;
    }
    
    pub fn set_remote_user(&mut self, play: &Play, default_username: String) {
        // FIXME: get current logged in username or update docs
        match &play.ssh_user {
            Some(x) => { *self.remote_user.lock().unwrap() = Some(x.clone()) },
            None => { *self.remote_user.lock().unwrap() = Some(String::from("root")); }
        }
    }

    pub fn get_remote_user(&mut self, host: String) -> String {
        let default = self.remote_user.lock().unwrap();
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