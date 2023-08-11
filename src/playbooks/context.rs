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

use std::sync::Mutex;
use std::sync::Arc;
use crate::util::io::{path_as_string,directory_as_string};
use crate::playbooks::language::Play;
use std::path::PathBuf;

pub struct PlaybookContext {
    pub playbook_path: Arc<Mutex<Option<String>>>,
    pub playbook_directory: Arc<Mutex<Option<String>>>,
    pub play: Arc<Mutex<Option<String>>>,
    pub task: Arc<Mutex<Option<String>>>,
    pub host: Arc<Mutex<Option<String>>>,
    pub all_hosts: Arc<Mutex<Option<Vec<String>>>>,
    pub role_path: Arc<Mutex<Option<String>>>,
    pub role_name: Arc<Mutex<Option<String>>>,
    pub remote_user: Arc<Mutex<Option<String>>
}

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
    
    pub set_remote_user(&mut self, play: &Play) {
        match play.remote_user {
            Some(x) => { *self.remote_user.lock().unwrap() = Some(x.clone()) },
            None => { *self.remote_user.lock().unwrap() = String::from("root"); }
        }
    }
}