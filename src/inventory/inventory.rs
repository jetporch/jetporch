use std::path::{Path,PathBuf};
use Vec;
use serde::{Deserialize};
use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::util::yaml::{show_yaml_error_in_context};
use crate::inventory::groups::{has_group, store_group, store_subgroup, store_group_variables};
use crate::inventory::hosts::{has_host, store_host, store_host_variables};

// ==============================================================================================================
// YAML SPEC
// ==============================================================================================================

// for groups/<groupname> inventory files
// for groups/<groupname> inventory files

//#[derive(Debug, PartialEq, Deserialize)]
#[derive(Debug,Deserialize)]
pub struct YamlGroup {
    hosts : Option<Vec<String>>,
    subgroups : Option<Vec<String>>
}

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================


pub fn load_inventory(inventory_paths: Vec<PathBuf>) -> Result<(), String> {

    store_group(String::from("all"));

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

// ==============================================================================================================
// PRIVATE INTERNALS
// ==============================================================================================================


fn load_classic_inventory_tree(include_groups: bool, path: &Path) -> Result<(), String> {
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
        
        // serialize the vars again just to make them easier to store/output elsewhere
        // this will also remove any comments and shorten things up
        let yaml_string = &serde_yaml::to_string(&yaml_result).unwrap();
        match is_group {
            true => store_group_variables(base_name.clone(), yaml_string.clone()),
            false => store_host_variables(base_name.clone(), yaml_string.clone())
        }

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

