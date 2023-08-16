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
use std::sync::Mutex;
use crate::inventory::groups::Group;

pub struct Host {
    pub name : String,
    pub variables : String,
    pub groups : Mutex<HashMap<String,Arc<Group>>>
}

impl Host {

    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            variables: String::new(),
            groups: Mutex::new(HashMap::new())
        }
    }

    // ==============================================================================================================
    // PUBLIC API - most code can use this
    // ==============================================================================================================
  
    pub fn get_groups(&self) -> Vec<Arc<Group>> {
        return self.groups.lock().unwrap().iter().map(|(k,v)| Arc::clone(&v)).collect();
    }

    pub fn add_group(&mut self, group: Arc<Group>) {
        self.groups.lock().unwrap().insert(group.name.clone(), Arc::clone(&group));
    }

    pub fn get_ancestor_groups(&self) -> Vec<Arc<Group>> {
        let mut results : HashMap<String, Arc<Group>> = HashMap::new();
        for g in self.get_groups().iter() {
            results.insert(g.name.clone(), Arc::clone(&g));
            for gp in g.get_ancestor_groups() { results.insert(gp.name.clone(), Arc::clone(&gp)); }
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
        for ancestor in self.get_ancestor_groups().iter() {
            let theirs = ancestor.get_variables();
            blended = blend_variables(&theirs.clone(), &blended.clone());
        }
        let mine = self.get_variables();
        return blend_variables(&mine.clone(), &blended.clone());
    }

}
