use crate::util::terminal::{two_column_table, captioned_display};
//use crate::inventory::hosts::{};
use crate::inventory::groups::{has_group, get_ancestor_groups, get_parent_groups, get_child_groups, get_descendant_groups,
    get_child_hosts, get_descendant_hosts, get_group_variables};

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

    if !has_group(group_name.clone()) {
        return Err(format!("no such group: {}", group_name.clone()));
    }
    
    println!("Report for Group: {}", group_name);
    println!("\n");

    //banner(format!("group: {}", group_name.clone()));

    let descendants      = get_descendant_groups(group_name.clone());
    let children         = get_child_groups(group_name.clone());
    let ancestors        = get_ancestor_groups(group_name.clone());
    let parents          = get_parent_groups(group_name.clone());
    let descendant_hosts = get_descendant_hosts(group_name.clone());
    let child_hosts      = get_child_hosts(group_name.clone());
    let group_variables  = get_group_variables(group_name.clone());

    //let parents      = get_all_group_parents(group_name);
    //let direct_hosts = get_all_group_hosts(group_name);
    //let variables    = get_group_variables(group_name);
    //let blended      = get_blended_group_variables(group_name);

    let descendant_hosts_count    = String::from(format!("{}", descendant_hosts.len()));
    let child_hosts_count = String::from(format!("{}", child_hosts.len()));
    
    // FIXME:
    
    let descendants_string = descendants.join(", ");
    let children_string = children.join(", ");
    let ancestors_string = ancestors.join(", ");
    let parents_string = parents.join(", ");
    let descendant_hosts_string = descendant_hosts.join(", ");
    let child_hosts_string = child_hosts.join(", ");

    // FIXME:

    let blended_variables = String::from("Bob?"); // FIXME
    

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

    /*
    let subgroups = get_all_group_subgroups(group_name);
    let ancestors = get_all_group_ancestors(group_name);
    let hosts = get_all_group_hosts(group_name);
    */

    two_column_table(
        String::from("Group Report:"), 
        String::from(""), 
        group_elements
    );
    println!("");

    two_column_table(
        String::from("Host Report:"), 
        String::from(""), 
        host_elements
    );
    println!("");

    captioned_display(
        String::from("Configured Variables"), 
        group_variables
    );
    println!("");

    captioned_display(
        String::from("Blended Variables"),
        blended_variables
    );
    println!("");
    return Ok(());
}




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
