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
use crate::inventory::groups::Group;
use std::sync::RwLock;
use serde_yaml;
//use crate::tasks::request::TaskRequest;
//use crate::tasks::response::TaskResponse;
use serde_yaml::Value::Mapping;


pub struct Host {
    pub name : String,
    pub variables : serde_yaml::Mapping,
    pub groups : HashMap<String, Arc<RwLock<Group>>>,
    //pub history: Vec<(Arc<TaskRequest>,Arc<TaskResponse>)>,
}

impl Host {

    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            variables : serde_yaml::Mapping::new(),
            groups: HashMap::new(),
            //history: Vec::new(),
        }
    }

    // ==============================================================================================================
    // PUBLIC API - most code can use this
    // ==============================================================================================================
  
    pub fn get_groups(&self) -> HashMap<String, Arc<RwLock<Group>>> {
        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.groups.iter() {
            results.insert(k.clone(), Arc::clone(&v));
        }
        return results;
    }

    pub fn get_group_names(&self) -> Vec<String> {
        return self.get_groups().iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn add_group(&mut self, name: &String, group: Arc<RwLock<Group>>) {
        self.groups.insert(name.clone(), Arc::clone(&group));
    }

    pub fn get_ancestor_groups(&self, depth_limit: usize) -> HashMap<String, Arc<RwLock<Group>>> {

        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.get_groups().into_iter() {
            results.insert(k, Arc::clone(&v));
            for (k2,v2) in v.read().unwrap().get_ancestor_groups(depth_limit).into_iter() { 
                results.insert(k2, Arc::clone(&v2)); 
            }
        }
        return results;
    }

    pub fn get_ancestor_group_names(&self) -> Vec<String> {
        return self.get_ancestor_groups(20usize).iter().map(|(k,_v)| k.clone()).collect();
    }

    pub fn get_variables(&self) -> serde_yaml::Mapping {
        return self.variables.clone();
    }

    pub fn set_variables(&mut self, variables: serde_yaml::Mapping) {
        self.variables = variables.clone();
    }

    pub fn get_blended_variables(&self) -> serde_yaml::Mapping {
        let mut blended : serde_yaml::Value = serde_yaml::Value::from(serde_yaml::Mapping::new());
        let ancestors = self.get_ancestor_groups(20);
        for (_k,v) in ancestors.iter() {
            let theirs : serde_yaml::Value = serde_yaml::Value::from(v.read().unwrap().get_variables());
            blend_variables(&mut blended, theirs);
        }
        let mine = serde_yaml::Value::from(self.get_variables());
        blend_variables(&mut blended, mine);
        return match blended {
            serde_yaml::Value::Mapping(x) => x,
            _ => panic!("get_blended_variables produced a non-mapping (1)")
        }
    }

    pub fn get_variables_yaml(&self) -> Result<String, String> {
        let result = serde_yaml::to_string(&self.get_variables());
        return match result {
            Ok(x) => Ok(x),
            Err(y) => Err(String::from("error loading variables"))
        }
    }

    pub fn get_blended_variables_yaml(&self) -> Result<String,String> {
        let result = serde_yaml::to_string(&self.get_blended_variables());
        return match result {
            Ok(x) => Ok(x),
            Err(y) => Err(String::from("error loading blended variables"))
        }
    }

}
