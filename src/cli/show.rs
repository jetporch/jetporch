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

use crate::util::terminal::{two_column_table, captioned_display};
use std::sync::Arc;
use std::sync::RwLock;
use crate::inventory::inventory::Inventory;

// cli support for the show-inventory subcommand

fn string_slice(values: &Vec<String>) -> String {
    // if there are too many values the output of various group/host lists in the tables
    // stops being useful. we may want to have some flag where we don't show the
    // nice tables for this, though right now they really don't exist
    if values.len() > 500 {
        let tmp = values[0..499].to_vec();
        return format!("{}, ...", tmp.join(", "));
    }
    return values.join(", ");
}

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

// jetp show --inventory <path> --hosts host1:host2

pub fn show_inventory_host(inventory: &Arc<RwLock<Inventory>>, host_name: &String) -> Result<(),String> {

    let inventory = inventory.read().expect("inventory read");

    if !inventory.has_host(&host_name.clone()) {
        return Err(format!("no such host: {}", host_name.clone()));
    }
    let binding = inventory.get_host(&host_name.clone());
    let host = binding.read().unwrap();
    
    println!("Host: {}", host_name);
    println!(" ");

    let mut parents               : Vec<String> = host.get_group_names();
    let mut ancestors             : Vec<String> = host.get_ancestor_group_names();
    let blended_variables     = host.get_blended_variables_yaml()?;
    
    parents.sort();
    ancestors.sort();

    let ancestor_string = string_slice(&ancestors);
    let parents_string  = string_slice(&parents);
  
    let host_elements : Vec<(String,String)> = vec![
        (String::from("Ancestor Groups"), ancestor_string),
        (String::from("Direct Groups"), parents_string),

    ];

    two_column_table(&String::from("Host Report:"), &String::from(""), &host_elements);
    println!("");

    captioned_display(&String::from("Variables"), &blended_variables);
    println!("");

    return Ok(());
}

// jetp show --inventory <path> # implicit --group all
// jetp show --inventory <path> --groups group1:group2

pub fn show_inventory_group(inventory: &Arc<RwLock<Inventory>>, group_name: &String) -> Result<(),String> {

    let inventory = inventory.read().expect("inventory read");

    if !inventory.has_group(&group_name.clone()) {
        return Err(format!("no such group: {}", group_name));
    }
    let binding = inventory.get_group(&group_name.clone());
    let group = binding.read().unwrap();
    
    println!("Group: {}", group_name);
    println!("");

    let mut descendants          : Vec<String>  = group.get_descendant_group_names();
    let mut children             : Vec<String>  = group.get_subgroup_names();
    let mut ancestors            : Vec<String>  = group.get_ancestor_group_names();
    let mut parents              : Vec<String>  = group.get_parent_group_names();
    let mut descendant_hosts     : Vec<String>  = group.get_descendant_host_names();
    let mut child_hosts          : Vec<String>  = group.get_direct_host_names();

    descendants.sort();
    children.sort();
    ancestors.sort();
    parents.sort();
    descendant_hosts.sort();
    child_hosts.sort();

    let blended_variables      = group.get_blended_variables_yaml()?;
    let descendant_hosts_count = String::from(format!("{}", descendant_hosts.len()));
    let child_hosts_count      = String::from(format!("{}", child_hosts.len()));
    
    // TODO: add a method that "..."'s these strings if too long - just use for hosts

    let descendants_string = string_slice(&descendants);
    let children_string = string_slice(&children);
    let ancestors_string = string_slice(&ancestors);
    let parents_string = string_slice(&parents);
    let descendant_hosts_string = string_slice(&descendant_hosts);
    let child_hosts_string = string_slice(&child_hosts);

    let group_elements : Vec<(String,String)> = vec![
        (String::from("All Descendants"), descendants_string),
        (String::from("Children"), children_string),
        (String::from("All Ancestors"), ancestors_string),
        (String::from("Parents"), parents_string)
    ];

    let host_elements : Vec<(String, String)> = vec![
        (format!("All Ancestors ({})",descendant_hosts_count), descendant_hosts_string),
        (format!("Children ({})", child_hosts_count), child_hosts_string),
    ];

    two_column_table(&String::from("Group Report:"), &String::from(""), &group_elements);
    println!("");

    
    two_column_table(&String::from("Host Report:"), &String::from(""), &host_elements);
    println!("");
    captioned_display(&String::from("Variables"), &blended_variables);
    println!("");

    return Ok(());
}



