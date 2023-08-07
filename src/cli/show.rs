use crate::util::terminal::{banner, two_column_table};
use crate::inventory::hosts::{};
use crate::inventory::groups::{get_all_group_parents, get_group_subgroups, get_all_group_hosts};

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================


pub fn show_inventory_host(host_name: String) -> Result<(),String> {


    //banner(format!("host: {}", host_name.clone()));


    /*
    if !has_host(host_name.clone()) {
        return Err(format!("group not found in inventory: {}", host_name));
    }

    banner_print("FIXME: will finish soon!"));
    */
    Ok(())
}

pub fn show_inventory_group(group_name: String) -> Result<(),String> {
    

    //banner(format!("group: {}", group_name.clone()));

    //let subgroups    = get_all_group_subgroups(group_name);
    //let parents      = get_all_group_parents(group_name);
    //let direct_hosts = get_all_group_hosts(group_name);
    //let variables    = get_group_variables(group_name);
    //let blended      = get_blended_group_variables(group_name);

    let hosts_immediate_count = String::from("42");
    let hosts_subgroup_count  = String::from("42");
    

    // FIXME:
    let subgroups_string = String::from("Bob?");
    let parents_string = String::from("Bob?");
    let hosts_string = String::from("Bob?");
    let variable_string = String::from("Bob?");
    let blended_string = String::from("Bob?");

    let elements : Vec<(String,String)> = vec![
        (String::from("Sub groups"), subgroups_string),
        (String::from("Parent groups"), parents_string),
        (String::from(format!("Hosts ({})", hosts_immediate_count)), hosts_string),
        (String::from(format!("Subgroup Hosts ({})", hosts_subgroup_count)), String::from("<not shown>")),
        (String::from("Variables (configured)"), variable_string),
        (String::from("Variables (evaluated)"), blended_string),
    ];

    /*
    let subgroups = get_all_group_subgroups(group_name);
    let ancestors = get_all_group_ancestors(group_name);
    let hosts = get_all_group_hosts(group_name);
    */

    two_column_table(String::from("Group"), group_name.clone(), elements);



    /*
    // FIXME: make a function called banner_print

    if !has_group(group_name.clone()) {
        return Err(format!("group not found in inventory: {}", group_name));
    }

    banner_print(format!("group: {}", group_name.clone()));
    */
                 
    //inventory_tree(group_name.clone(), 0);

    // FIXME: finish the details table here
    // FIXME: banner table method that takes a key_order and a hashmap of key/values

    //let keys : Vec<String> = [
    //    "parent groups": 
    //];
    //Vec!

    /*
    let mut buffer = String::new();

    // FIXME: move to inventory? can we split group and host APIs into files?

    let parent_groups_str : String = get_all_subgroups(group_name.clone())
        .iter()
        .map(|s| s.to_string())
        .reduce(|cur: String, nxt: String| cur + ",".to_string + &nxt);


        .collect()
        .fold("".to_string(), )

    println("begin details table here: FIXME")
    */
    Ok(())
    

}
