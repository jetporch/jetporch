
use once_cell::sync::Lazy;
use std::sync::Mutex;
use Vec;
use std::collections::{HashMap,HashSet};
use crate::inventory::hosts::{associate_host_to_group, has_host, create_host};

static GROUPS          : Lazy<Mutex<HashSet<String>>>                 = Lazy::new(||Mutex::new(HashSet::new()));
static GROUP_SUBGROUPS : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_PARENTS   : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_HOSTS     : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));
static GROUP_VARIABLES : Lazy<Mutex<HashMap<String,String>>>          = Lazy::new(||Mutex::new(HashMap::new()));

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

// is this group name in inventory?
pub fn has_group(group_name: String) -> bool {
    return GROUPS.lock().expect("LOCKED").contains(&group_name);
}


//get_ancestor_groups, get_parent_groups, get_child_groups, get_descendent_groups, get_child_hosts, get_descendent_hosts}
pub fn get_ancestor_groups(group: String) -> Vec<String> {
    let set : Vec<String> = internal_get_all_group_parents(group.clone(), 0usize).into_iter().collect();
    return set
}

pub fn get_parent_groups(group: String) -> Vec<String> {
    println!("FIXME: not implemented1");
    return Vec::new();
}

pub fn get_child_groups(group: String) -> Vec<String> {
    println!("FIXME: not implemented2");
    return Vec::new();
}

pub fn get_descendant_groups(group: String) -> Vec<String> {
    println!("FIXME: not implemented2");
    return Vec::new();
}

pub fn get_child_hosts(group: String) -> Vec<String> {
    println!("FIXME: not implemented2");
    return Vec::new();
}

pub fn get_descendant_hosts(group: String) -> Vec<String> {
    println!("FIXME: not implemented2");
    return Vec::new();
}



// ==============================================================================================================
// PACKAGE API (for use by inventory.rs/hosts.rs only)
// ==============================================================================================================

// add a child group, used by inventory loading code
pub fn store_subgroup(group: String, child: String) {
    if !has_group(group.clone()) { create_group(group.clone()); }
    if !has_group(child.clone()) { create_group(child.clone()); }
    associate_subgroup(group, child);
}

pub fn store_group_variables(group: String, yaml_string: String) {
    let mut vars = GROUP_VARIABLES.lock().unwrap();
    let vars_entry: &mut String = vars.get_mut(&group).unwrap();
    vars_entry.clear();
    vars_entry.push_str(&yaml_string.clone());
}

pub fn store_group(group: String) {
    create_group(group.clone());
}

pub fn associate_host(group: String, host: String) {
    if !has_host(host.clone()) {
        create_host(host.clone());
    }
    if !has_group(group.clone()) {
        create_group(group.clone());
    }
    let group = group.clone();
    let mut group_hosts = GROUP_HOSTS.lock().unwrap();
    let group_hosts_entry: &mut HashSet<std::string::String> = group_hosts.get_mut(&group).unwrap();
    group_hosts_entry.insert(host.clone());

    associate_host_to_group(group, host);
}

// ==============================================================================================================
// PRIVATE INTERNALS
// ==============================================================================================================

fn create_group(group_name: String) {

    assert!(!has_group(group_name.clone()));
    let mut groups          = GROUPS.lock().unwrap();
    let mut group_parents   = GROUP_PARENTS.lock().unwrap();
    let mut group_subgroups = GROUP_SUBGROUPS.lock().unwrap();
    let mut group_variables = GROUP_VARIABLES.lock().unwrap();
    let mut group_hosts     = GROUP_HOSTS.lock().unwrap();

    groups.insert(group_name.clone());
    group_subgroups.insert(group_name.clone(), HashSet::new());
    group_variables.insert(group_name.clone(), String::from(""));
    group_hosts.insert(group_name.clone(), HashSet::new());    

    group_parents.insert(group_name.clone(), HashSet::new());
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

fn internal_get_all_group_parents(group: String, depth: usize) -> Vec<String> {
    if depth > 1000 {
        panic!("maximum group depth (1000) exceeded: {}", depth);
    }
    let group = group.clone();
    let group_parents = GROUP_PARENTS.lock().unwrap();
    let mut group_parents_entry = group_parents.get(&group).unwrap();
    let mut group_names : Vec<String> = group_parents_entry.iter().map(|x| x.clone()).collect();
    std::mem::drop(group_parents);
    
    let mut results: Vec<String> = Vec::new();

    for parent in group_names.iter() {

        let grand_parents = internal_get_all_group_parents(parent.clone(), depth + 1);
        for grand_parent in grand_parents.iter() {
            if ! results.contains(&grand_parent.clone()) {
                results.push(grand_parent.clone());
            }
        }
        if ! results.contains(&parent.clone()) {
            results.push(parent.clone());
        }
    }

    let mut seen: HashSet<String> = HashSet::new();
    results.retain(|&x| seen.contains(&x) == false);
    return results;
}






