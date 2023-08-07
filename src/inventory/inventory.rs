use once_cell::sync::Lazy;
use std::path::{Path,PathBuf};
use std::sync::Mutex;
use Vec;
use serde::{Deserialize};
use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::util::yaml::{show_yaml_error_in_context};
use std::collections::{HashMap,HashSet};
//use serde_yaml::Value;
//use std::sync::Arc;

//bookmark - we are debugging the makefile to make sure this constructs hosts correctly
// next up we should make an iterator that takes a list of groups and returns all the hosts therein
// and then build the show command
// after that, parse variable files
// after that, dynamic inventory

// when ready the playbook parser can work a little similar to this,  keeping data structures
// with names and numeric IDs that load the YAML structures as things go.
// we could even store their YAML representations in the structures after first parse
// and then re-walk that structure, TBD, if it was too much to keep the structs in memory.

//=========================================================================================================
// the inventory is fairly mutable, hopefully the playbook tree will be simpler
//=========================================================================================================


static GROUPS          : Lazy<Mutex<HashSet<String>>>                 = Lazy::new(||Mutex::new(HashSet::new()));
static GROUP_SUBGROUPS : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_PARENTS   : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_HOSTS     : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_VARIABLES : Lazy<Mutex<HashMap<String,String>>>          = Lazy::new(||Mutex::new(HashMap::new()));
static HOSTS           : Lazy<Mutex<HashSet<String>>>                 = Lazy::new(||Mutex::new(HashSet::new()));
static HOST_VARIABLES  : Lazy<Mutex<HashMap<String,String>>>          = Lazy::new(||Mutex::new(HashMap::new()));
static HOST_GROUPS     : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));

#[derive(Debug, PartialEq, Deserialize)]
pub struct YamlGroup {
    hosts : Option<Vec<String>>,
    subgroups : Option<Vec<String>>
}

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
    host_variables.insert(host_name.clone(), String::from(""));
    let mut host_groups = HOST_GROUPS.lock().unwrap();//.expect("LOCKED");
    host_groups.insert(host_name.clone(), HashSet::new());
}

fn create_group(group_name: String) {
    println!(">>>>>>>> CREATE GROUP [{}] <<<<<<<<", group_name);

    assert!(!has_group(group_name.clone()));
    let mut groups = GROUPS.lock().unwrap();
    let mut group_parents = GROUP_PARENTS.lock().unwrap();
    let mut group_subgroups = GROUP_SUBGROUPS.lock().unwrap();
    let mut group_variables = GROUP_VARIABLES.lock().unwrap();
    let mut group_hosts     = GROUP_HOSTS.lock().unwrap();

    groups.insert(group_name.clone());
    group_subgroups.insert(group_name.clone(), HashSet::new());
    group_variables.insert(group_name.clone(), String::from(""));
    group_hosts.insert(group_name.clone(), HashSet::new());    

    group_parents.insert(group_name.clone(), HashSet::new());
    //println!("adding parent to {}", group_name.clone());
    group_subgroups.insert(group_name.clone(), HashSet::new());
    if !group_name.eq(&String::from("all")) {

        std::mem::drop(groups);
        std::mem::drop(group_parents);
        std::mem::drop(group_subgroups);
        std::mem::drop(group_variables);
        std::mem::drop(group_hosts);
        associate_subgroup(String::from("all"), group_name);
    }
}

fn store_host(group_name: String, host_name: String) {
    if !(has_host(host_name.clone())) {
        create_host(host_name.clone());
    }
    associate_host(group_name, host_name);

}

fn associate_host(group: String, host: String) {
    if !has_host(host.clone()) {
        create_host(host.clone());
    }
    if !has_group(group.clone()) {
        create_group(group.clone());
    }
    let group = group.clone();
    let mut group_hosts = GROUP_HOSTS.lock().expect("LOCKED");
    let mut host_groups = HOST_GROUPS.lock().expect("LOCKED");
    let group_hosts_entry: &mut HashSet<std::string::String> = group_hosts.get_mut(&group).unwrap();
    let host_groups_entry: &mut HashSet<std::string::String> = host_groups.get_mut(&host).unwrap();
    group_hosts_entry.insert(host.clone());
    host_groups_entry.insert(group.clone());
}

fn associate_subgroup(group: String, child: String) {
    let group = group.clone();
    let child = child.clone();
    if !has_group(group.clone()) { create_group(group.clone()); }
    if !has_group(child.clone()) { create_group(child.clone()); }
    let mut group_subgroups = GROUP_SUBGROUPS.lock().unwrap();
    let mut group_parents = GROUP_PARENTS.lock().unwrap();
    let group_subgroups_entry: &mut HashSet<std::string::String> = group_subgroups.get_mut(&group).unwrap();
    let group_parents_entry: &mut HashSet<std::string::String> = group_parents.get_mut(&child).unwrap();
    group_subgroups_entry.insert(child.clone());
    group_parents_entry.insert(group.clone());
}

fn store_subgroup(group: String, child: String) {
    if !has_group(group.clone()) { create_group(group.clone()); }
    if !has_group(child.clone()) { create_group(child.clone()); }
    associate_subgroup(group, child);
}

pub fn internal_get_all_group_parents(group: String, depth: usize) -> Vec<String> {
    if depth > 1000 {
        panic!("maximum group depth (1000) exceeded: {}", depth);
    }
    let group = group.clone();
    println!("fetching group parents for {}", group);
    let group_parents = GROUP_PARENTS.lock().unwrap();
    let mut group_parents_entry = group_parents.get(&group).unwrap();
    let mut group_names : Vec<String> = group_parents_entry.iter().map(|x| x.clone()).collect();
    std::mem::drop(group_parents);
    
    let mut results: Vec<String> = Vec::new();

    for parent in group_names.iter() {
        let grand_parents = internal_get_all_group_parents(parent.clone(), depth + 1);
        for grand_parent in grand_parents.iter() {
            results.push(grand_parent.clone())
        }
        results.push(parent.clone());
    }
    return results;
}

pub fn get_all_group_parents(group: String) -> HashSet<String> {
    let mut set : HashSet<String> = internal_get_all_group_parents(group.clone(), 0usize).into_iter().collect();
    return set
}

pub fn internal_get_all_host_groups(host: String) -> Vec<String> {
    let host = host.clone();
    let host_groups = HOST_GROUPS.lock().unwrap();
    let host_groups_entry = host_groups.get(&host).unwrap();
    let mut group_names: Vec<String> = host_groups_entry.iter().map(|x| x.clone()).collect();
    std::mem::drop(host_groups);

    let mut results : Vec<String> = Vec::new();

    for group in group_names.iter() {
        results.push(group.clone());
        let parents = internal_get_all_group_parents(group.clone(), 0); 
        for parent in parents.iter() {
            results.push(parent.clone());
        }
    }
    return results;
}

pub fn get_all_host_groups(host: String) -> HashSet<String> {
    let mut set : HashSet<String> = internal_get_all_host_groups(host.clone()).into_iter().collect();
    return set
}

pub fn load_inventory(inventory_paths: Vec<PathBuf>) -> Result<(), String> {

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
    return Ok(())
}

pub fn load_classic_inventory_tree(include_groups: bool, path: &Path) -> Result<(), String> {
    let path_buf           = PathBuf::from(path);
    let group_vars_pathbuf = path_buf.join("group_vars");
    let host_vars_pathbuf  = path_buf.join("host_vars");
    let groups_path        = path_buf.join("groups");
    let group_vars_path    = group_vars_pathbuf.as_path();
    let host_vars_path     = host_vars_pathbuf.as_path();
      
    if include_groups {
        load_groups_directory(&groups_path)?;
    }
    if group_vars_path.exists() {
        load_vars_directory(&group_vars_path, true)?;
    }
    if host_vars_path.exists() {
        load_vars_directory(&host_vars_path, false)?;
    }
    return Ok(())
}


fn load_groups_directory(path: &Path) -> Result<(), String> {
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

    let hosts = &yaml_group.hosts;
    if hosts.is_some() {
        let hosts = hosts.as_ref().unwrap();
        for hostname in hosts {
            store_host(group_name.clone(), hostname.clone());
        }
    }
    let subgroups = &yaml_group.subgroups;
    if subgroups.is_some() {
        let subgroups = subgroups.as_ref().unwrap();
        for subgroupname in subgroups {
            store_subgroup(group_name.clone(), subgroupname.clone());
        }
    }

}
              
fn load_vars_directory(path: &Path, is_group: bool) -> Result<(), String> {

    path_walk(path, |vars_path| {

        let base_name = path_basename_as_string(&vars_path).clone();
        // FIXME: warning and continue instead?
        match is_group {
            true => {
                // FIXME warning/logging library?
                if !has_group(base_name.clone()) { 
                    println!("warning: attempting to define group_vars for a group not in inventory: {}", base_name); 
                    return Ok(());
                }
            } false => {
                // FIXME warning/logging library?
                if !has_host(base_name.clone()) { 
                    println!("warning: attempting to define host_vars for a host not in inventory: {}", base_name); 
                    return Ok(());
                }
            }
        }
        
        let file = jet_file_open(&vars_path)?;
        let file_parse_result: Result<serde_yaml::Mapping, serde_yaml::Error> = serde_yaml::from_reader(file);
        if file_parse_result.is_err() {
             show_yaml_error_in_context(&file_parse_result.unwrap_err(), &vars_path);
             return Err(format!("edit the file and try again?"));
        } 
        let yaml_result = file_parse_result.unwrap();
        let mut vars = match is_group {
            true  => GROUP_VARIABLES.lock().unwrap(),
            false => HOST_VARIABLES.lock().unwrap()
        };
        let vars_entry: &mut String = vars.get_mut(&base_name).unwrap();
        vars_entry.clear();
        vars_entry.push_str(&serde_yaml::to_string(&yaml_result).unwrap());

        Ok(())
    })?;
    Ok(())
}

// TODO: blended yaml results per host ... but only those selected in the play.

fn load_dynamic_inventory(path: &Path) -> Result<(), String> {
    println!("load_dynamic_inventory: NOT IMPLEMENTED");

    // FIXME: implement the script execution/parsing parts
    load_classic_inventory_tree(false, &path)?;
    //return Err(format!("NOT IMPLEMENTED3: {}", path.display()));
    Err("load  dynamic inventory is not implemented".to_string())
}

