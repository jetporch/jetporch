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

use std::collections::HashMap;
use crate::util::yaml::blend_variables;
use std::sync::Arc;
use crate::inventory::groups::Group;
use std::sync::RwLock;
use std::collections::HashSet;
use serde_yaml;

#[derive(Clone,Copy,Debug)]
pub enum HostOSType {
    Linux,
    MacOS,
}

pub struct Host {
    pub name               : String,
    pub groups             : HashMap<String, Arc<RwLock<Group>>>,
    pub variables          : serde_yaml::Mapping,
    pub os_type            : Option<HostOSType>,
    checksum_cache         : HashMap<String,String>,
    checksum_cache_task_id : usize,
    facts                  : serde_yaml::Value,
    dyn_variables          : serde_yaml::Value,
    notified_handlers      : HashMap<usize, HashSet<String>>
}

impl Host {

    pub fn new(name: &String) -> Self {
        Self {
            name: name.clone(),
            variables : serde_yaml::Mapping::new(),
            groups: HashMap::new(),
            os_type: None,
            checksum_cache: HashMap::new(),
            checksum_cache_task_id: 0,
            facts: serde_yaml::Value::from(serde_yaml::Mapping::new()),
            dyn_variables: serde_yaml::Value::from(serde_yaml::Mapping::new()),
            notified_handlers: HashMap::new()
        }
    }

    pub fn notify(&mut self, play_number: usize, signal: &String) {
        if ! self.notified_handlers.contains_key(&play_number) {
            self.notified_handlers.insert(play_number, HashSet::new());
        }
        let entry = self.notified_handlers.get_mut(&play_number).unwrap();
        entry.insert(signal.clone());
    }

    pub fn is_notified(&self, play_number: usize, signal: &String) -> bool {
        let entry = self.notified_handlers.get(&play_number);
        if entry.is_none() {
            return false;
        } else {
            return entry.unwrap().contains(&signal.clone());
        }
    }

    pub fn set_checksum_cache(&mut self, path: &String, checksum: &String) {
        self.checksum_cache.insert(path.clone(), checksum.clone());
    }

    pub fn get_checksum_cache(&mut self, task_id: usize, path: &String) -> Option<String> {
        if task_id > self.checksum_cache_task_id {
            self.checksum_cache_task_id = task_id;
            self.checksum_cache.clear();
        }
        if self.checksum_cache.contains_key(path) {
            let result = self.checksum_cache.get(path).unwrap();
            return Some(result.clone());
        }
        else {
            return None;
        }
    }

    // used by connection class on initial connect
    pub fn set_os_info(&mut self, uname_output: &String) -> Result<(),String> {
        if uname_output.starts_with("Linux") {
            self.os_type = Some(HostOSType::Linux);
        } else if uname_output.starts_with("Darwin") {
            self.os_type = Some(HostOSType::MacOS);
        } else {
            return Err(format!("OS Type could not be detected from uname -a: {}", uname_output));
        }
        return Ok(());
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

    #[inline(always)]
    pub fn get_group_names(&self) -> Vec<String> {
        return self.get_groups().iter().map(|(k,_v)| k.clone()).collect();
    }

    #[inline(always)]
    pub fn add_group(&mut self, name: &String, group: Arc<RwLock<Group>>) {
        self.groups.insert(name.clone(), Arc::clone(&group));
    }

    pub fn get_ancestor_groups(&self, depth_limit: usize) -> HashMap<String, Arc<RwLock<Group>>> {

        let mut results : HashMap<String, Arc<RwLock<Group>>> = HashMap::new();
        for (k,v) in self.get_groups().into_iter() {
            results.insert(k, Arc::clone(&v));
            for (k2,v2) in v.read().expect("group read").get_ancestor_groups(depth_limit).into_iter() { 
                results.insert(k2, Arc::clone(&v2)); 
            }
        }
        return results;
    }

    #[inline(always)]
    pub fn get_ancestor_group_names(&self) -> Vec<String> {
        return self.get_ancestor_groups(20usize).iter().map(|(k,_v)| k.clone()).collect();
    }

    #[inline(always)]
    pub fn get_variables(&self) -> serde_yaml::Mapping {
        return self.variables.clone();
    }

    #[inline(always)]
    pub fn set_variables(&mut self, variables: serde_yaml::Mapping) {
        self.variables = variables.clone();
    }

    #[inline(always)]
    pub fn update_variables(&mut self, mapping: serde_yaml::Mapping) {
        let map = mapping.clone();
        blend_variables(&mut self.dyn_variables, serde_yaml::Value::Mapping(map));
    }

    pub fn get_blended_variables(&self) -> serde_yaml::Mapping {
        let mut blended : serde_yaml::Value = serde_yaml::Value::from(serde_yaml::Mapping::new());
        let ancestors = self.get_ancestor_groups(20);
        for (_k,v) in ancestors.iter() {
            let theirs : serde_yaml::Value = serde_yaml::Value::from(v.read().unwrap().get_variables());
            blend_variables(&mut blended, theirs);
        }
        blend_variables(&mut blended, self.dyn_variables.clone());
        let mine = serde_yaml::Value::from(self.get_variables());
        blend_variables(&mut blended, mine);
        blend_variables(&mut blended, self.facts.clone());
        return match blended {
            serde_yaml::Value::Mapping(x) => x,
            _ => panic!("get_blended_variables produced a non-mapping (1)")
        }
    }

    pub fn update_facts(&mut self, mapping: &Arc<RwLock<serde_yaml::Mapping>>) {
        let map = mapping.read().unwrap().clone();
        blend_variables(&mut self.facts, serde_yaml::Value::Mapping(map));
    }

    pub fn get_variables_yaml(&self) -> Result<String, String> {
        let result = serde_yaml::to_string(&self.get_variables());
        return match result {
            Ok(x) => Ok(x),
            Err(_y) => Err(String::from("error loading variables"))
        }
    }

    pub fn get_blended_variables_yaml(&self) -> Result<String,String> {
        let result = serde_yaml::to_string(&self.get_blended_variables());
        return match result {
            Ok(x) => Ok(x),
            Err(_y) => Err(String::from("error loading blended variables"))
        }
    }

}
