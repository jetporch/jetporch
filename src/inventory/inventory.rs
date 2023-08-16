
use std::sync::Mutex;
use std::collections::{HashMap};
use std::sync::Arc;
use crate::inventory::hosts::Host;
use crate::inventory::groups::Group;

pub struct Inventory {
    pub groups : Mutex<HashMap<String, Arc<Group>>>,
    pub hosts  : Mutex<HashMap<String, Arc<Host>>>
}

impl Inventory {

    pub fn new() -> Self {
        Self {
            groups : Mutex::new(HashMap::new()),
            hosts  : Mutex::new(HashMap::new())
        }
    }
    
    pub fn has_group(&self, group_name: &String) -> bool {
        let guard = self.groups.lock().unwrap();
        return guard.contains_key(&group_name.clone());
    }

    pub fn get_group(&self, group_name: &String) -> Arc<Group> {
        let guard = self.groups.lock().unwrap();
        let arc = guard.get(&group_name.clone()).unwrap();
        return Arc::clone(&arc); 
    }

    pub fn has_host(&self, host_name: &String) -> bool {
        let guard = self.hosts.lock().unwrap();
        return guard.contains_key(&host_name.clone());
    }

    pub fn get_host(&self, host_name: &String) -> Arc<Host> {
        let guard = self.hosts.lock().unwrap();
        return Arc::clone(guard.get(&host_name.clone()).unwrap());
    }

    // ==============================================================================================================
    // PACKAGE API (for use by loading.rs only)
    // ==============================================================================================================

    pub fn store_subgroup(&mut self, group_name: &String, subgroup_name: &String) {
        if self.has_group(&group_name.clone()) { self.create_group(&group_name.clone()); }
        if !self.has_group(&subgroup_name.clone()) { self.create_group(&subgroup_name.clone()); }
        self.associate_subgroup(&group_name.clone(), &subgroup_name.clone());
    }

    pub fn store_group_variables(&mut self, group_name: &String, yaml_string: &String) {
        let group = self.get_group(&group_name.clone());
        group.set_variables(&yaml_string.clone());
    }

    pub fn store_group(&mut self, group: &String) {
        self.create_group(&group.clone()); 
    }

    pub fn associate_host(&mut self, group_name: &String, host: Arc<Host>) {
        if !self.has_host(&host.name.clone()) { panic!("host does not exist"); }
        if !self.has_group(&group_name.clone()) { self.create_group(&group_name.clone()); }
        let group_obj = self.get_group(&group_name.clone());
        group_obj.hosts.get_mut().unwrap().insert(host.name.clone(), Arc::clone(&host));
        self.associate_host_to_group(&group_name.clone(), &host.name.clone());
    }

    pub fn associate_host_to_group(&self, group_name: &String, host_name: &String) {
        let host = self.get_host(&host_name.clone());
        let group = self.get_group(&group_name.clone());
        host.add_group(Arc::clone(&group));
        group.add_host(Arc::clone(&host));
    }

    pub fn store_host_variables(&self, host_name: &String, yaml_string: &String) {
        let host = self.get_host(&host_name.clone());
        host.set_variables(&yaml_string.clone());
    }

    pub fn create_host(&mut self, host_name: &String) {
        assert!(!self.has_host(&host_name.clone()));
        self.hosts.get_mut().unwrap().insert(host_name.clone(), Arc::new(Host::new(&host_name.clone())));
    }

    pub fn store_host(&mut self, group_name: &String, host_name: &String) {
        if !(self.has_host(&host_name.clone())) {
            self.create_host(&host_name.clone());
        }
        let host = self.get_host(host_name);
        self.associate_host(&group_name.clone(), Arc::clone(&host));
    }

    // ==============================================================================================================
    // PRIVATE INTERNALS
    // ==============================================================================================================

    fn create_group(&mut self, group_name: &String) {
        assert!(!self.has_group(&group_name.clone()));
        self.groups.get_mut().unwrap().insert(group_name.clone(), Arc::new(Group::new(&group_name.clone())));
        if !group_name.eq(&String::from("all")) {
            self.associate_subgroup(&String::from("all"), &group_name);
        }
    }

    fn associate_subgroup(&self, group_name: &String, subgroup_name: &String) {
        if !self.has_group(&group_name.clone()) { self.create_group(&group_name.clone()); }
        if !self.has_group(&subgroup_name.clone()) { self.create_group(&subgroup_name.clone()); }
        let group = self.get_group(&group_name);
        let subgroup = self.get_group(&subgroup_name);
        group.add_subgroup(Arc::clone(&subgroup));
        subgroup.add_parent(Arc::clone(&group));

    }


}