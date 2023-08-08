use once_cell::sync::Lazy;
use std::sync::Mutex;
use Vec;
use std::collections::{HashMap,HashSet};
use crate::inventory::groups::{associate_host, get_group_variables, get_group_ancestor_groups};
use crate::util::data::{deduplicate,recursive_descent};
use crate::util::yaml::{blend_variables};

static HOSTS           : Lazy<Mutex<HashSet<String>>>                 = Lazy::new(||Mutex::new(HashSet::new()));
static HOST_VARIABLES  : Lazy<Mutex<HashMap<String,String>>>          = Lazy::new(||Mutex::new(HashMap::new()));
static HOST_GROUPS     : Lazy<Mutex<HashMap<String,HashSet<String>>>> = Lazy::new(||Mutex::new(HashMap::new()));

// ==============================================================================================================
// PUBLIC API - most code can use this
// ==============================================================================================================

pub fn get_host_groups(host: String) -> Vec<String> {
    let host = host.clone();
    let host_groups = HOST_GROUPS.lock().unwrap();
    let host_groups_entry = host_groups.get(&host).unwrap();
    let groups: Vec<String> = host_groups_entry.iter().map(|x| x.clone()).collect();
    return groups;
}

pub fn get_host_ancestor_groups(host: String) -> Vec<String> {
    let mut results : HashSet<String> = HashSet::new();
    for g in get_host_groups(host.clone()) {
        results.insert(g.clone());
        for gp in get_group_ancestor_groups(g.clone()) { results.insert(gp.clone()); }
    }
    let result2: Vec<String> = results.iter().map(|x| x.clone()).collect();
    return result2;
}

pub fn has_host(host_name: String) -> bool {
    return HOSTS.lock().unwrap().contains(&host_name);
}

pub fn get_host_variables(host: String) -> String {
    let vars = HOST_VARIABLES.lock().unwrap();
    let vars_entry: &String = vars.get(&host).unwrap();
    return vars_entry.clone()
}

pub fn get_host_blended_variables(host: String) -> String {
    let mut blended = String::from("");
    let ancestors = get_host_ancestor_groups(host.clone());
    for ancestor in ancestors.iter() {
        let theirs = get_group_variables(ancestor.clone());
        blended = blend_variables(theirs.clone(), blended.clone());
    }
    let mine = get_host_variables(host.clone());
    return blend_variables(mine.clone(), blended.clone());
}


// =============================================================================================================
// INVENTORY API - for use by inventory.rs/groups.rs only!
// =============================================================================================================

pub fn store_host(group_name: String, host_name: String) {
    if !(has_host(host_name.clone())) {
        create_host(host_name.clone());
    }
    associate_host(group_name, host_name);
}

// =============================================================================================================
// PACKAGE API - fairly low level - for use by groups.rs only!
// =============================================================================================================

pub fn associate_host_to_group(group: String, host: String) {
    let host = host.clone();
    let mut host_groups = HOST_GROUPS.lock().unwrap();
    let host_groups_entry: &mut HashSet<std::string::String> = host_groups.get_mut(&host).unwrap();
    host_groups_entry.insert(group.clone());
}

pub fn store_host_variables(host: String, yaml_string: String) {
    let mut vars = HOST_VARIABLES.lock().unwrap();
    let vars_entry: &mut String = vars.get_mut(&host).unwrap();
    vars_entry.clear();
    vars_entry.push_str(&yaml_string.clone());
}

pub fn create_host(host_name: String) {
    assert!(!has_host(host_name.clone()));
    let mut hosts = HOSTS.lock().unwrap();
    hosts.insert(host_name.clone());

    let mut host_variables = HOST_VARIABLES.lock().unwrap();
    host_variables.insert(host_name.clone(), String::from(""));
    let mut host_groups = HOST_GROUPS.lock().unwrap();
    host_groups.insert(host_name.clone(), HashSet::new());
}

// ==============================================================================================================
// PRIVATE INTERNALS
// ==============================================================================================================

// ...



