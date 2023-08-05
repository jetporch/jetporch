use std::path::{Path,PathBuf};
use Vec;
use std::sync::Arc;
use serde::{Deserialize};
use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::util::yaml::{show_yaml_error_in_context};
use crate::inventory::group::Group;
use crate::inventory::host::Host;

#[derive(Debug, PartialEq, Deserialize)]
pub struct YamlGroup {
    hosts : Option<Vec<String>>,
    subgroups : Option<Vec<String>>
}
//#[serde(tag = "Group")]

// FIXME: we'll need this later for other things
//#[serde(flatten)] f
//extras: HashMap<String, String>,

pub struct Inventory {   
    pub groups: Vec<Arc<Group>>,
    pub hosts: Vec<Arc<Host>>
}

impl Inventory  {

    pub fn new() -> Self {
        Inventory { 
            groups: Vec::new(),
            hosts: Vec::new(),
        }
    }

    pub fn find_host(&mut self, host_name: String) -> Option<&mut Arc<Host>> {
        //let needle = host_name.clone()
        return self.hosts.iter_mut().find(|x| x.name.eq(&host_name));
    }

    pub fn find_group(&mut self, group_name: String) -> Option<&mut Arc<Group>> {
        return self.groups.iter_mut().find(|x| x.name.eq(&group_name));
    }

    pub fn find_or_create_host(&mut self, host_name: String) -> Arc<Host> {
        //let hosts = &mut self.hosts; 
        match self.find_host(host_name.clone()) {
            Some(host) => Arc::clone(host),
            _ => {
                let ptr = Arc::new(Host::new(host_name.clone()));
                self.hosts.push(Arc::clone(&ptr));
                return Arc::clone(&ptr);
            }
        }
    }

    pub fn find_or_create_group(&mut self, group_name: String) -> Arc<Group> {
        //let groups = &mut self.groups;   
        match self.find_group(group_name.clone()) {
            Some(group) => Arc::clone(group),
            _ => {
                let ptr = Arc::new(Group::new(group_name.clone()));
                self.groups.push(Arc::clone(&ptr));
                return Arc::clone(&ptr);
            }
        }
    }
   
    pub fn load_inventory(&mut self, inventory_paths: Vec<PathBuf>) -> Result<(), String> {
        for inventory_path_buf in inventory_paths {
            let inventory_path = inventory_path_buf.as_path();
            if inventory_path.is_dir() {
                let groups_pathbuf      = inventory_path_buf.join("groups");
                let groups_path         = groups_pathbuf.as_path();
                if groups_path.exists() && groups_path.is_dir() {
                    self.load_classic_inventory_tree(true, &inventory_path)?;
                } else {
                    if is_executable(&inventory_path) {
                        self.load_dynamic_inventory(&inventory_path)?;
                    } else {
                        return Err(
                            format!("non-directory path to --inventory ({}) is not executable", 
                                inventory_path.display()))
                    }    
                }
            }
        }
        // FIXME: need to do cycle detection
        return Ok(())
    }

    pub fn load_classic_inventory_tree(&mut self, include_groups: bool, path: &Path) -> Result<(), String> {
        
        let path_buf = PathBuf::from(path);
        let group_vars_pathbuf = path_buf.join("group_vars");
        let host_vars_pathbuf  = path_buf.join("host_vars");
        let groups_path        = path_buf.join("groups");
        let group_vars_path    = group_vars_pathbuf.as_path();
        let host_vars_path     = host_vars_pathbuf.as_path();
        
        self.find_or_create_group(String::from("all"));

        if include_groups {
            self.load_groups(&groups_path)?;
        }
        if group_vars_path.exists() {
            self.load_group_vars(&group_vars_path)?;
        }
        if host_vars_path.exists() {
            self.load_host_vars(&host_vars_path)?;
        }
        return Ok(())
    }


    fn load_groups(&mut self, path: &Path) -> Result<(), String> {
       
        path_walk(path, |groups_file_path| {

            let group_name = path_basename_as_string(&groups_file_path).clone();
            let groups_file = jet_file_open(&groups_file_path)?;
            let groups_file_parse_result: Result<YamlGroup, serde_yaml::Error> = serde_yaml::from_reader(groups_file);
            
            if groups_file_parse_result.is_err() {
                show_yaml_error_in_context(&groups_file_parse_result.unwrap_err(), &groups_file_path);
                return Err(format!("edit the file and try again?"));
            } 
            
            let yaml_result = groups_file_parse_result.unwrap();
            self.add_group_file_contents_to_inventory(
                group_name.clone(), &yaml_result
            );
            
            Ok(())

        })?;

        Ok(())
    }

    fn store_host(&mut self, group_ptr: &mut Arc<Group>, host_name: String) {
        self.find_or_create_host(host_name.clone());
        let mutable_group = Arc::get_mut(group_ptr).unwrap();
        mutable_group.add_host(host_name.clone());
    }

    fn store_subgroup(&mut self, group_ptr: &mut Arc<Group>, sub_group_name: String) {
        self.find_or_create_group(sub_group_name.clone());
        let mutable_group = Arc::get_mut(group_ptr).unwrap();
        mutable_group.add_subgroup(sub_group_name.clone());
    }

    fn add_group_file_contents_to_inventory(&mut self, group_name: String, yaml_group: &YamlGroup) {
        
        let group_ptr : Arc<Group> = self.find_or_create_group(group_name);

        let hosts = &yaml_group.hosts;
        if hosts.is_some() {
            let hosts = hosts.as_ref().unwrap();
            for hostname in hosts {
                self.store_host(&mut Arc::clone(&group_ptr), hostname.clone());
            }
        }

        let subgroups = &yaml_group.subgroups;
        if subgroups.is_some() {
            let subgroups = subgroups.as_ref().unwrap();
            for subgroupname in subgroups {
                self.store_subgroup(&mut Arc::clone(&group_ptr), subgroupname.clone());
            }
        }

    }
              
    fn load_group_vars(&mut self, path: &Path) -> Result<(), String> {
        return Err(format!("NOT IMPLEMENTED1: {}", path.display()));
    }

    fn load_host_vars(&mut self, path: &Path) -> Result<(), String> {
       // FIXME -- walk this path and load each file
       return Err(format!("NOT IMPLEMENTED2: {}", path.display()));
    }
    
    fn load_dynamic_inventory(&mut self, path: &Path) -> Result<(), String> {
        // FIXME: implement the script execution/parsing parts
        self.load_classic_inventory_tree(false, &path)?;
        return Err(format!("NOT IMPLEMENTED3: {}", path.display()));
    }

}