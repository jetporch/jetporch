
use std::collections::{HashMap};
use std::sync::Arc;
use crate::inventory::hosts::Host;
use crate::inventory::groups::Group;
use std::sync::RwLock;

pub struct Inventory {
    pub groups : HashMap<String, Arc<RwLock<Group>>>,
    pub hosts  : HashMap<String, Arc<RwLock<Host>>>,
    // SSH inventory is not required to have a localhost in it but needs the object
    // regardless, this is returned if it is not in inventory so we always get the same
    // object.
    backup_localhost: Arc<RwLock<Host>>
}

impl Inventory {

    pub fn new() -> Self {
        Self {
            groups : HashMap::new(),
            hosts  : HashMap::new(),
            backup_localhost: Arc::new(RwLock::new(Host::new(&String::from("localhost"))))
        }
    }
    
    #[inline]
    pub fn has_group(&self, group_name: &String) -> bool {
        return self.groups.contains_key(&group_name.clone());
    }

    pub fn get_group(&self, group_name: &String) -> Arc<RwLock<Group>> {
        let arc = self.groups.get(group_name).unwrap();
        return Arc::clone(&arc); 
    }

    #[inline]
    pub fn has_host(&self, host_name: &String) -> bool {
        return self.hosts.contains_key(host_name);
    }

    #[inline]
    pub fn get_host(&self, host_name: &String) -> Arc<RwLock<Host>> {

        // an explicit fetch of a host is sometimes performed by the connection plugin
        // which does not bother with the has_host check. If localhost is not in inventory
        // we don't need any variables from it.

        if self.has_host(host_name) {
            let host = self.hosts.get(host_name).unwrap();
            return Arc::clone(&host);
        }
        else if host_name.eq("localhost") {
            return Arc::clone(&self.backup_localhost);
        } else {
            panic!("internal error: code should call has_host before get_host");
        }
    }

    // ==============================================================================================================
    // PACKAGE API (for use by loading.rs only)
    // ==============================================================================================================

    pub fn store_subgroup(&mut self, group_name: &String, subgroup_name: &String) {
        if self.has_group(group_name) { self.create_group(group_name); }
        if !self.has_group(subgroup_name) { self.create_group(subgroup_name); }
        self.associate_subgroup(group_name, subgroup_name);
    }

    #[inline]
    pub fn store_group_variables(&mut self, group_name: &String, mapping: serde_yaml::Mapping) {
        let group = self.get_group(group_name);
        group.write().expect("group write").set_variables(mapping);
    }

    #[inline]
    pub fn store_group(&mut self, group: &String) {
        self.create_group(&group.clone()); 
    }

    #[inline]
    pub fn associate_host(&mut self, group_name: &String, host_name: &String, host: Arc<RwLock<Host>>) {
        if !self.has_host(&host_name) { panic!("host does not exist"); }
        if !self.has_group(&group_name) { self.create_group(group_name); }
        let group_obj = self.get_group(group_name);
        // FIXME: these add method should all take strings, not all are consistent yet?
        group_obj.write().unwrap().add_host(&host_name.clone(), host);
        self.associate_host_to_group(&group_name.clone(), &host_name.clone());
    }

    #[inline]
    pub fn associate_host_to_group(&self, group_name: &String, host_name: &String) {
        let host = self.get_host(host_name);
        let group = self.get_group(group_name);
        host.write().expect("host write").add_group(group_name, Arc::clone(&group));
        group.write().expect("group write").add_host(host_name, Arc::clone(&host));
    }

    #[inline]
    pub fn store_host_variables(&mut self, host_name: &String, mapping: serde_yaml::Mapping) {
        let host = self.get_host(host_name);
        host.write().unwrap().set_variables(mapping);
    }

    #[inline]
    pub fn create_host(&mut self, host_name: &String) {
        assert!(!self.has_host(host_name));
        self.hosts.insert(host_name.clone(), Arc::new(RwLock::new(Host::new(&host_name.clone()))));
    }

    #[inline]
    pub fn store_host(&mut self, group_name: &String, host_name: &String) {
        if !(self.has_host(&host_name)) {
            self.create_host(&host_name);
        }
        let host = self.get_host(host_name);
        self.associate_host(group_name, host_name, Arc::clone(&host));
    }

    // ==============================================================================================================
    // PRIVATE INTERNALS
    // ==============================================================================================================

    fn create_group(&mut self, group_name: &String) {
        if self.has_group(group_name) {
            return;
        }
        self.groups.insert(group_name.clone(), Arc::new(RwLock::new(Group::new(&group_name.clone()))));
        if !group_name.eq(&String::from("all")) {
            self.associate_subgroup(&String::from("all"), &group_name);
        }
    }

    fn associate_subgroup(&mut self, group_name: &String, subgroup_name: &String) {
        if !self.has_group(&group_name.clone()) { self.create_group(&group_name.clone()); }
        if !self.has_group(&subgroup_name.clone()) { self.create_group(&subgroup_name.clone()); }
        {
            let group = self.get_group(group_name);
            let subgroup = self.get_group(subgroup_name);
            group.write().expect("group write").add_subgroup(subgroup_name, Arc::clone(&subgroup));
        }
        {
            let group = self.get_group(group_name);
            let subgroup = self.get_group(subgroup_name);
            subgroup.write().expect("subgroup write").add_parent(group_name, Arc::clone(&group));
        }
    }


}