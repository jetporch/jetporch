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

use std::collections::{HashMap};
use crate::util::yaml::{blend_variables};
use std::sync::Arc;
use crate::inventory::hosts::Host;
use std::sync::RwLock;

pub struct Group {
    pub name : String,
    pub subgroups : HashMap<String, Arc<RwLock<Self>>>,
    pub parents : HashMap<String, Arc<RwLock<Self>>>,
    pub hosts : HashMap<String, Arc<RwLock<Host>>>,
    pub variables : String
}

impl Group {

    pub fn new(name: &String) -> Self {
        Self {
            name : name.clone(),
            subgroups : HashMap::new(),
            parents : HashMap::new(),
            hosts : HashMap::new(),
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

    pub fn add_subgroup(&mut self, name: &String, subgroup: Arc<RwLock<Group>>) {
        assert!(!name.eq(&self.name));
        self.subgroups.insert(
            name.clone(), 
            Arc::clone(&subgroup)
        );
    }

    pub fn add_host(&mut self, name: &String, host: Arc<RwLock<Host>>) {
        self.hosts.insert(
            name.clone(), 
            Arc::clone(&host)
        );
    }

    pub fn add_parent(&mut self, name: &String, parent: Arc<RwLock<Group>>) {
        assert!(!name.eq(&self.name));
        self.parents.insert(
            name.clone(), 
            Arc::clone(&parent)
        );
    }

    pub fn get_ancestor_groups(&self, depth_limit: usize) -> HashMap<String, Arc<RwLock<Group>>> {
        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.parents.iter() {
            results.insert(k.clone(), Arc::clone(v));
            if depth_limit > 0 {
                for (k2,v2) in v.read().unwrap().get_ancestor_groups(depth_limit-1) { 
                    results.insert(k2.clone(),Arc::clone(&v2));
                }
            }
        }
        return results;
    }

    pub fn get_ancestor_group_names(&self) -> Vec<String> {
        return self.get_ancestor_groups(10usize).iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_descendant_groups(&self, depth_limit: usize) -> HashMap<String, Arc<RwLock<Group>>> {

        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.subgroups.iter() {
            if results.contains_key(&k.clone()) {
                continue;
            }
            if depth_limit > 0 {
                for (k2,v2) in v.read().unwrap().get_descendant_groups(depth_limit-1).iter() { 
                    results.insert(
                        k2.clone(), 
                        Arc::clone(&v2)
                    ); 
                }
            }
            results.insert(
                k.clone(), 
                Arc::clone(&v)
            );
        }
        return results;
    }

    pub fn get_descendant_group_names(&self) -> Vec<String> {
        return self.get_descendant_groups(10usize).iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_parent_groups(&self) -> HashMap<String, Arc<RwLock<Group>>> {
        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.parents.iter() {
            results.insert(
                k.clone(), 
                Arc::clone(&v)
            );
        }
        return results;
    }

    pub fn get_parent_group_names(&self) -> Vec<String> {
        return self.get_parent_groups().iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_subgroups(&self) -> HashMap<String, Arc<RwLock<Group>>> {
        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.subgroups.iter() {
            results.insert(
                k.clone(), 
                Arc::clone(&v)
            );
        }
        return results;
    }

    pub fn get_subgroup_names(&self) -> Vec<String> {
        return self.get_subgroups().iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_direct_hosts(&self) -> HashMap<String, Arc<RwLock<Host>>> {
        let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
        for (k,v) in self.hosts.iter() {
            results.insert(
                k.clone(), 
                Arc::clone(&v)
            );
        }
        return results;
    }

    pub fn get_direct_host_names(&self) -> Vec<String> {
        return self.get_direct_hosts().iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_descendant_hosts(&self) -> HashMap<String, Arc<RwLock<Host>>> {
        let mut results : HashMap<String, Arc<RwLock<Host>>> = HashMap::new();
        let children = self.get_direct_hosts();
        for (k,v) in children { results.insert(k.clone(), Arc::clone(&v));  }
        let groups = self.get_descendant_groups(20usize);
        for (_k,v) in groups.iter() {
            let hosts = v.read().unwrap().get_direct_hosts();
            for (k2,v2) in hosts.iter() { results.insert(k2.clone(), Arc::clone(&v2));  }
        }   
        return results;
    }

    pub fn get_descendant_host_names(&self) -> Vec<String> {
        return self.get_descendant_hosts().iter().map(|(k,_v)| k.clone()).collect();
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
        let ancestors = self.get_ancestor_groups(20);
        for (_k,v) in ancestors.iter() {
            let theirs = v.read().unwrap().get_variables();
            blended = blend_variables(&theirs.clone(), &blended.clone());
        }
        let mine = self.get_variables();
        return blend_variables(&mine.clone(), &blended.clone());
    }

}








