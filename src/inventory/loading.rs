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

use std::path::{Path,PathBuf};
use Vec;
use serde::{Deserialize};
use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::util::yaml::{show_yaml_error_in_context};
use crate::inventory::inventory::Inventory;
use std::sync::Arc;
use std::sync::RwLock;

// ==============================================================================================================
// YAML SPEC
// ==============================================================================================================
// for groups/<groupname> inventory files

//#[derive(Debug, PartialEq, Deserialize)]
#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlGroup {
    hosts : Option<Vec<String>>,
    subgroups : Option<Vec<String>>,
    ssh_user : Option<String>,
    ssh_port: Option<String>,
}

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

pub fn load_inventory(inventory: &Arc<RwLock<Inventory>>, inventory_paths: Arc<RwLock<Vec<PathBuf>>>) -> Result<(), String> {

    {
        let mut inv_obj = inventory.write().unwrap();
        inv_obj.store_group(&String::from("all"));
    }

    for inventory_path_buf in inventory_paths.read().unwrap().iter() {
        let inventory_path = inventory_path_buf.as_path();
        if inventory_path.is_dir() {
            let groups_pathbuf      = inventory_path_buf.join("groups");
            let groups_path         = groups_pathbuf.as_path();
            if groups_path.exists() && groups_path.is_dir() {
                load_on_disk_inventory_tree(inventory, true, &inventory_path)?;
            } else {
                if is_executable(&inventory_path) {
                    load_dynamic_inventory(inventory, &inventory_path)?;
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

// loads an entire on-disk inventory tree structure (groups/, group_vars/, host_vars/)
fn load_on_disk_inventory_tree(inventory: &Arc<RwLock<Inventory>>, include_groups: bool, path: &Path) -> Result<(), String> {
    let path_buf           = PathBuf::from(path);
    let group_vars_pathbuf = path_buf.join("group_vars");
    let host_vars_pathbuf  = path_buf.join("host_vars");
    let groups_path        = path_buf.join("groups");
    let group_vars_path    = group_vars_pathbuf.as_path();
    let host_vars_path     = host_vars_pathbuf.as_path();
      
    if include_groups {
        load_groups_directory(inventory, &groups_path)?;
    }
    if group_vars_path.exists() {
        load_vars_directory(inventory, &group_vars_path, true)?;
    }
    if host_vars_path.exists() {
        load_vars_directory(inventory, &host_vars_path, false)?;
    }
    return Ok(())
}

// for inventory/groups/* files
fn load_groups_directory(inventory: &Arc<RwLock<Inventory>>, path: &Path) -> Result<(), String> {
    path_walk(path, |groups_file_path| {
        let group_name = path_basename_as_string(&groups_file_path).clone();
        let groups_file = jet_file_open(&groups_file_path)?;
        let groups_file_parse_result: Result<YamlGroup, serde_yaml::Error> = serde_yaml::from_reader(groups_file);
        if groups_file_parse_result.is_err() {
            show_yaml_error_in_context(&groups_file_parse_result.unwrap_err(), &groups_file_path);
            return Err(format!("edit the file and try again?"));
        }   
        let yaml_result = groups_file_parse_result.unwrap();
        add_group_file_contents_to_inventory(inventory, group_name.clone(), &yaml_result);
        Ok(())
    })?;
    Ok(())
}


// for inventory/groups/* files
fn add_group_file_contents_to_inventory(inventory: &Arc<RwLock<Inventory>>, group_name: String, yaml_group: &YamlGroup) {
    let mut inventory = inventory.write().unwrap();
    let hosts = &yaml_group.hosts;
    if hosts.is_some() {
        let hosts = hosts.as_ref().unwrap();
        for hostname in hosts { inventory.store_host(&group_name.clone(), &hostname.clone()); }
    }
    let subgroups = &yaml_group.subgroups;
    if subgroups.is_some() {
        let subgroups = subgroups.as_ref().unwrap();
        for subgroupname in subgroups {
            // FIXME: we should not panic here, but do something better
            if !group_name.eq(subgroupname) {
                inventory.store_subgroup(&group_name.clone(), &subgroupname.clone()); 
            }
        }
    }

}
            
// this is used by both on-disk and dynamic inventory sources to load group/ and vars/ directories
fn load_vars_directory(inventory: &Arc<RwLock<Inventory>>, path: &Path, is_group: bool) -> Result<(), String> {

    let inv = inventory.write().unwrap();

    path_walk(path, |vars_path| {


        let base_name = path_basename_as_string(&vars_path).clone();
        // FIXME: warning and continue instead?
        match is_group {
            true => {
                if !inv.has_group(&base_name.clone()) { return Ok(()); }
            } false => {
                if !inv.has_host(&base_name.clone()) { return Ok(()); }
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
        //let yaml_string = &serde_yaml::to_string(&yaml_result).unwrap();
        match is_group {
            true  => {
                let group = inv.get_group(&base_name.clone());
                group.write().unwrap().set_variables(yaml_result);
            }
            false => {
                let host = inv.get_host(&base_name);
                host.write().unwrap().set_variables(yaml_result);
            }
        }
        Ok(())
    })?;
    Ok(())
}

// TODO: implement
fn load_dynamic_inventory(inventory: &Arc<RwLock<Inventory>>, path: &Path) -> Result<(), String> {
     println!("load_dynamic_inventory: NOT IMPLEMENTED");
    // FIXME: implement the script execution/parsing parts on top
    load_on_disk_inventory_tree(inventory, false, &path)?;
    //return Err(format!("NOT IMPLEMENTED3: {}", path.display()));
    Err("load  dynamic inventory is not implemented".to_string())
}

