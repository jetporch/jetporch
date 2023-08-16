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
use std::collections::{HashMap};
use crate::util::yaml::{blend_variables};
use std::sync::Arc;
use crate::inventory::hosts::Host;

pub struct Group {
    pub name : String,
    pub subgroups : Mutex<HashMap<String, Arc<Self>>>,
    pub parents : Mutex<HashMap<String, Arc<Self>>>,
    pub hosts : Mutex<HashMap<String, Arc<Host>>>,
    pub variables : String
}

impl Group {

    pub fn new(name: &String) -> Self {
        Self {
            name : name.clone(),
            subgroups : Mutex::new(HashMap::new()),
            parents : Mutex::new(HashMap::new()),
            hosts : Mutex::new(HashMap::new()),
            variables :String::new()
        }
    }

    /*
    pub fn has_group(&self, &group_name: String) -> bool {
        let guard = self.groups.lock().unwrap();
        return guard.contains_key(&group_name.clone());
    }

    pub fn get_group(&self, &group_name: String) -> Arc<Group> {
        let guard = self.groups.lock().unwrap();
        let arc = groups.get(&group_name.clone()).unwrap();
        return Arc::clone(&arc);    
    }
    */

    pub fn add_subgroup(&mut self, subgroup: Arc<Group>) {
        self.subgroups.lock().unwrap().insert(subgroup.name.clone(), Arc::clone(&subgroup));
    }

    pub fn add_host(&mut self, host: Arc<Host>) {
        self.hosts.lock().unwrap().insert(host.name.clone(), Arc::clone(&host));
    }

    pub fn add_parent(&mut self, parent: Arc<Group>) {
        self.parents.lock().unwrap().insert(parent.name.clone(), Arc::clone(&parent));
    }

    pub fn get_ancestor_groups(&self) -> Vec<Arc<Group>> {
        let mut results : HashMap<String, Arc<Group>> = HashMap::new();
        for (k,v) in self.parents.lock().unwrap().iter() {
            results.insert(k.clone(), Arc::clone(&v));
            for recursed in v.get_ancestor_groups() { results.insert(recursed.name.clone(), Arc::clone(&recursed)); }
        }
        return results.iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn get_descendant_groups(&self) -> Vec<Arc<Group>> {
        let mut results : HashMap<String, Arc<Group>> = HashMap::new();
        for (k,v) in self.subgroups.lock().unwrap().iter() {
            results.insert(k.clone(), Arc::clone(&v));
            for recursed in v.get_descendant_groups().iter() { results.insert(recursed.name.clone(), Arc::clone(&recursed)); }
        }
        return results.iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn get_parent_groups(&self) -> Vec<Arc<Group>> {
        return self.parents.lock().unwrap().iter().map(|(k,v)| Arc::clone(&v)).collect()
    }

    pub fn get_subgroups(&self) -> Vec<Arc<Group>> {
        return self.subgroups.lock().unwrap().iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn get_direct_hosts(&self) -> Vec<Arc<Host>> {
        return self.hosts.lock().unwrap().iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn get_descendant_hosts(&self) -> Vec<Arc<Host>> {
        let mut results : HashMap<String, Arc<Host>> = HashMap::new();
        let children = self.get_direct_hosts();
        for ch in children { results.insert(ch.name.clone(), Arc::clone(&ch));  }
        let groups = self.get_descendant_groups();
        for group in groups.iter() {
            let hosts = group.get_direct_hosts();
            for host in hosts.iter() { results.insert(host.name.clone(), Arc::clone(&host));  }
        }   
        return results.iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn get_variables(&self) -> String {
        return self.variables.clone();
    }

    pub fn set_variables(&mut self, yaml_string: &String) {
        self.variables.clear();
        self.variables.push_str(&yaml_string.clone());
    }

    pub fn get_blended_variables(&self) -> String {
        let mut blended = String::from("");
        let ancestors = self.get_ancestor_groups();
        for ancestor in ancestors.iter() {
            let theirs = ancestor.get_variables();
            blended = blend_variables(&theirs.clone(), &blended.clone());
        }
        let mine = self.get_variables();
        return blend_variables(&mine.clone(), &blended.clone());
    }

}








