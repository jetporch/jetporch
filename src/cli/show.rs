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

// code for the CLI subcommand 'show'.

use crate::util::terminal::{two_column_table, captioned_display};
use std::sync::Mutex;
use std::sync::Arc;
use crate::inventory::inventory::Inventory;

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

// jetp show --inventory <path> --hosts host1:host2

pub fn show_inventory_host(inventory: &Arc<Mutex<Inventory>>, host_name: &String) -> Result<(),String> {

    let mut inventory = inventory.get_mut().unwrap();


    if !inventory.has_host(&host_name.clone()) {
        return Err(format!("no such host: {}", host_name.clone()));
    }
    let host = inventory.get_host(&host_name.clone());
    
    println!("Host: {}", host_name);
    println!("");

    let parents               : Vec<String> = host.get_groups().iter().map(|x| x.name.clone()).collect();
    let ancestors             : Vec<String> = host.get_ancestor_groups().iter().map(|x| x.name.clone()).collect();
    let host_variables        = host.get_variables();
    let blended_variables     = host.get_blended_variables();
    
    let ancestor_string = ancestors.join(", ");
    let parents_string  = parents.join(", ");
  
    let host_elements : Vec<(String,String)> = vec![
        (String::from("Ancestor Groups"), ancestor_string),
        (String::from("Direct Groups"), parents_string),

    ];

    two_column_table(String::from("Host Report:"), String::from(""), host_elements);
    println!("");

    captioned_display(String::from("Configured Variables"), host_variables);
    println!("");
    captioned_display(String::from("Blended Variables"), blended_variables);
    println!("");

    Ok(())
}

// jetp show --inventory <path> # implicit --group all
// jetp show --inventory <path> --groups group1:group2

pub fn show_inventory_group(inventory: &Arc<Mutex<Inventory>>, group_name: &String) -> Result<(),String> {

    let inventory = inventory.lock().unwrap();

    if !inventory.has_group(&group_name.clone()) {
        return Err(format!("no such group: {}", group_name));
    }
    let group = inventory.get_group(&group_name.clone());
    
    println!("Group: {}", group_name);
    println!("");

    let descendants          : Vec<String>  = group.get_descendant_groups().iter().map(|x| x.name.clone()).collect();
    let children             : Vec<String>  = group.get_subgroups().iter().map(|x| x.name.clone()).collect();
    let ancestors            : Vec<String>  = group.get_ancestor_groups().iter().map(|x| x.name.clone()).collect();
    let parents              : Vec<String>  = group.get_parent_groups().iter().map(|x| x.name.clone()).collect();
    let descendant_hosts     : Vec<String>  = group.get_descendant_hosts().iter().map(|x| x.name.clone()).collect();
    let child_hosts          : Vec<String>  = group.get_direct_hosts().iter().map(|x| x.name.clone()).collect();
    let group_variables        = group.get_variables();
    let blended_variables      = group.get_blended_variables();
    let descendant_hosts_count = String::from(format!("{}", descendant_hosts.len()));
    let child_hosts_count      = String::from(format!("{}", child_hosts.len()));
    
    // TODO: add a method that "..."'s these strings if too long - just use for hosts

    let descendants_string = descendants.join(", ");
    let children_string = children.join(", ");
    let ancestors_string = ancestors.join(", ");
    let parents_string = parents.join(", ");
    let descendant_hosts_string = descendant_hosts.join(", ");
    let child_hosts_string = child_hosts.join(", ");

    let group_elements : Vec<(String,String)> = vec![
        (String::from("All Descendants"), descendants_string),
        (String::from("Children"), children_string),
        (String::from("All Ancestors"), ancestors_string),
        (String::from("Parents"), parents_string)
    ];

    let host_elements : Vec<(String, String)> = vec![
        (String::from(format!("All Ancestors ({})",descendant_hosts_count)), descendant_hosts_string),
        (String::from(format!("Children ({})", child_hosts_count)), child_hosts_string),
    ];

    two_column_table(String::from("Group Report:"), String::from(""), group_elements);
    println!("");
    two_column_table(String::from("Host Report:"), String::from(""), host_elements);
    println!("");
    captioned_display(String::from("Configured Variables"), group_variables);
    println!("");
    captioned_display(String::from("Blended Variables"), blended_variables);
    println!("");

    return Ok(());
}



