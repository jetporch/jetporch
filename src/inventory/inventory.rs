use once_cell::sync::Lazy;
use std::path::{Path,PathBuf};
use std::sync::Mutex;
use Vec;
use serde::{Deserialize};
use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::util::yaml::{show_yaml_error_in_context};
use std::collections::{HashMap,HashSet};

//=========================================================================================================
// the inventory is fairly mutable, hopefully the playbook tree will be simpler
//=========================================================================================================

static GROUPS : Lazy<Mutex<HashSet<String>>> = Lazy::new(|| { 
    let m = HashSet::new();
    Mutex::new(m)
});

static GROUP_SUBGROUPS : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(|| { 
    let m = HashMap::new();
    Mutex::new(m)
});

static GROUP_PARENTS : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(|| { 
    let m =  HashMap::new();
    Mutex::new(m) 
});

static GROUP_HOSTS : Lazy<Mutex<HashMap<String,HashSet<String>>>> =  Lazy::new(|| { 
    let m =  HashMap::new();
    Mutex::new(m) 
});

static GROUP_VARIABLES : Lazy<Mutex<HashMap<String,serde_yaml::value::Mapping>>> = Lazy::new(|| { 
    let m =  HashMap::new();
    Mutex::new(m) 
});

static HOSTS : Lazy<Mutex<HashSet<String>>> = Lazy::new(|| { 
    let m = HashSet::new();
    Mutex::new(m) 
});

static HOST_VARIABLES : Lazy<Mutex<HashMap<String,serde_yaml::value::Mapping>>> = Lazy::new(|| { 
    let m =  HashMap::new();
    Mutex::new(m) 
});

static HOST_GROUPS : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(|| { 
    let m =  HashMap::new();
    Mutex::new(m) 
});

#[derive(Debug, PartialEq, Deserialize)]
pub struct YamlGroup {
    hosts : Option<Vec<String>>,
    subgroups : Option<Vec<String>>
}

//#[serde(tag = "Group")]
// FIXME: we'll need this later for other things
//#[serde(flatten)] f
//extras: HashMap<String, String>,

/*
fn with_state<R>(data: Lazy<Mutex<HashSet<String>>>, f: impl FnOnce(&mut HashSet<String>) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}
*/

pub fn has_host(host_name: String) -> bool {
    return HOSTS.lock().expect("LOCKED").contains(&host_name);
}

pub fn has_group(group_name: String) -> bool {
    return GROUPS.lock().expect("LOCKED").contains(&group_name);
}

fn create_host(host_name: String) {

    assert!(!has_host(host_name.clone()));

    let mut hosts = HOSTS.lock().unwrap();
    hosts.insert(host_name.clone());

    let mut host_variables = HOST_VARIABLES.lock().unwrap();
    host_variables.insert(host_name.clone(), serde_yaml::value::Mapping::new());
     
}

fn create_group(group_name: String) {
    println!("debug: creating group: {}", group_name);

    assert!(!has_group(group_name.clone()));

    //with_state(GROUPS, |groups| groups.insert(group_name.clone()));
    let mut groups = GROUPS.lock().unwrap(); // .expect("LOCKED");
    let mut group_parents = GROUP_PARENTS.lock().unwrap();//.expect("LOCKED");
    let mut group_subgroups = GROUP_SUBGROUPS.lock().unwrap();//.expect("LOCKED");
    let mut group_variables = GROUP_VARIABLES.lock().unwrap();//.expect("LOCKED");

    groups.insert(group_name.clone());
    group_subgroups.insert(group_name.clone(), HashSet::new());
    group_variables.insert(group_name.clone(), serde_yaml::value::Mapping::new());
        
    if !group_name.eq(&String::from("all")) {
        group_parents.insert(group_name.clone(), HashSet::new());
        group_subgroups.insert(group_name.clone(), HashSet::new());
        std::mem::drop(groups);
        std::mem::drop(group_parents);
        std::mem::drop(group_subgroups);
        std::mem::drop(group_variables);
        associate_subgroup(String::from("all"), group_name);
    }
}

fn store_host(group_name: String, host_name: String) {
    println!("SH");

    if !(has_host(host_name.clone())) {
        create_host(host_name.clone());
    }
    associate_host(group_name, host_name);
}

fn associate_host(group: String, host: String) {
    println!("AH");

    let group = group.clone();
    let mut group_hosts = GROUP_HOSTS.lock().expect("LOCKED");
    let mut host_groups = HOST_GROUPS.lock().expect("LOCKED");
    let group_hosts_entry: &mut HashSet<std::string::String> = group_hosts.get_mut(&group).unwrap();
    let host_groups_entry: &mut HashSet<std::string::String> = host_groups.get_mut(&group).unwrap();
    group_hosts_entry.insert(host.clone());
    host_groups_entry.insert(group.clone());
}

fn associate_subgroup(group: String, child: String) {
    println!("AS");

    let group = group.clone();
    let child = child.clone();
    println!("AS1");

    if !has_group(child.clone()) {
        create_group(child.clone());
    }

    let mut group_subgroups = GROUP_SUBGROUPS.lock().expect("LOCKED");
    println!("AS2");

    let mut group_parents = GROUP_PARENTS.lock().expect("LOCKED");
    println!("AS3: getting subgroups on: {}", group);

    let group_subgroups_entry: &mut HashSet<std::string::String> = group_subgroups.get_mut(&group).unwrap();
    let group_parents_entry: &mut HashSet<std::string::String> = group_parents.get_mut(&child).unwrap();
    group_subgroups_entry.insert(child.clone());
    group_parents_entry.insert(group.clone());

}

fn store_subgroup(group: String, child: String) {
    println!("SSG");

    if !has_group(group.clone()) {
        create_group(group.clone());
    }
    associate_subgroup(group, child);
}

pub fn load_inventory(inventory_paths: Vec<PathBuf>) -> Result<(), String> {
    println!("LI");

    create_group(String::from("all"));
    for inventory_path_buf in inventory_paths {
        let inventory_path = inventory_path_buf.as_path();
        if inventory_path.is_dir() {
            let groups_pathbuf      = inventory_path_buf.join("groups");
            let groups_path         = groups_pathbuf.as_path();
            if groups_path.exists() && groups_path.is_dir() {
                load_classic_inventory_tree(true, &inventory_path)?;
            } else {
                if is_executable(&inventory_path) {
                    load_dynamic_inventory(&inventory_path)?;
                } else {
                    return Err(
                        format!("non-directory path to --inventory ({}) is not executable", 
                            inventory_path.display()))
                }    
            }
        }
    }
    // FIXME: need to do cycle detection yet - doesn't exist in the datastructure but can exist logically
    return Ok(())
}

pub fn load_classic_inventory_tree(include_groups: bool, path: &Path) -> Result<(), String> {
    println!("LCIT");
    let path_buf = PathBuf::from(path);
    let group_vars_pathbuf = path_buf.join("group_vars");
    let host_vars_pathbuf  = path_buf.join("host_vars");
    let groups_path        = path_buf.join("groups");
    let group_vars_path    = group_vars_pathbuf.as_path();
    let host_vars_path     = host_vars_pathbuf.as_path();
      
    if include_groups {
        load_groups_directory(&groups_path)?;
    }
    if group_vars_path.exists() {
        load_group_vars_directory(&group_vars_path)?;
    }
    if host_vars_path.exists() {
        load_host_vars_directory(&host_vars_path)?;
    }
    return Ok(())
}


fn load_groups_directory(path: &Path) -> Result<(), String> {
    println!("LGD");

    path_walk(path, |groups_file_path| {

        let group_name = path_basename_as_string(&groups_file_path).clone();
        let groups_file = jet_file_open(&groups_file_path)?;
        let groups_file_parse_result: Result<YamlGroup, serde_yaml::Error> = serde_yaml::from_reader(groups_file);
            
        if groups_file_parse_result.is_err() {
            show_yaml_error_in_context(&groups_file_parse_result.unwrap_err(), &groups_file_path);
            return Err(format!("edit the file and try again?"));
        } 
            
        let yaml_result = groups_file_parse_result.unwrap();
        add_group_file_contents_to_inventory(
            group_name.clone(), &yaml_result
        );
            
        Ok(())
    })?;
    Ok(())
}



fn add_group_file_contents_to_inventory(group_name: String, yaml_group: &YamlGroup) {
        
    println!("GFCTI");

    let hosts = &yaml_group.hosts;
    if hosts.is_some() {
        let hosts = hosts.as_ref().unwrap();
        for hostname in hosts {
            println!("calling store on host: {}", hostname);
            store_host(group_name.clone(), hostname.clone());
        }
    }

    let subgroups = &yaml_group.subgroups;
    if subgroups.is_some() {
        let subgroups = subgroups.as_ref().unwrap();
        for subgroupname in subgroups {
            println!("calling store on subgroup: {}", subgroupname);
            store_subgroup(group_name.clone(), subgroupname.clone());
        }
    }

}
              
fn load_group_vars_directory(path: &Path) -> Result<(), String> {
    println!("L1");

    return Err(format!("NOT IMPLEMENTED1: {}", path.display()));
}

fn load_host_vars_directory(path: &Path) -> Result<(), String> {
    println!("L2");

   // FIXME -- walk this path and load each file
   return Err(format!("NOT IMPLEMENTED2: {}", path.display()));
}
    
fn load_dynamic_inventory(path: &Path) -> Result<(), String> {
    println!("L3");

    // FIXME: implement the script execution/parsing parts
    load_classic_inventory_tree(false, &path)?;
    return Err(format!("NOT IMPLEMENTED3: {}", path.display()));
}

