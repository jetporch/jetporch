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

use crate::connection::connection::{Connection};
use crate::inventory::hosts::Host;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::collections::HashMap;

pub struct ConnectionCache {
    connections: HashMap<String, Arc<Mutex<dyn Connection>>>
}

impl ConnectionCache {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new()
        }
    }

    pub fn add_connection(&mut self, host:&Arc<RwLock<Host>>, connection: &Arc<Mutex<dyn Connection>>) {
        let host2 = host.read().unwrap();
        self.connections.insert(host2.name.clone(), Arc::clone(connection));
    }

    pub fn has_connection(&self, host: &Arc<RwLock<Host>>) -> bool {
        let host2 = host.read().unwrap();
        return self.connections.contains_key(&host2.name.clone());
    }

    pub fn get_connection(&self, host: &Arc<RwLock<Host>>) -> Arc<Mutex<dyn Connection>> {
        let host2 = host.read().unwrap();
        return Arc::clone(self.connections.get(&host2.name.clone()).unwrap());
    }

    pub fn clear(&mut self) {
        self.connections.clear();
    }
}