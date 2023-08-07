use crate::util::terminal::{banner};
use crate::inventory::hosts::{has_host};
use crate::inventory::groups::{get_all_group_parents, get_group_subgroups, get_all_group_hosts};

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================


pub fn show_inventory_host(host_name: String) -> Result<(),String> {


    banner(format!("host: {}", host_name.clone()));


    /*
    if !has_host(host_name.clone()) {
        return Err(format!("group not found in inventory: {}", host_name));
    }

    banner_print("FIXME: will finish soon!"));
    */
    Ok(())
}

pub fn show_inventory_group(group_name: String) -> Result<(),String> {
    

    banner(format!("host: {}", group_name.clone()));


    /*
    // FIXME: make a function called banner_print

    if !has_group(group_name.clone()) {
        return Err(format!("group not found in inventory: {}", group_name));
    }

    banner_print(format!("group: {}", group_name.clone()));
    */
                 
    inventory_tree(group_name.clone(), 0);

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

// ==============================================================================================================
// PRIVATE INTERNALS
// ==============================================================================================================


fn inventory_tree(group_name: String, depth: usize) {

    /*
    let mut root = String::new("all");
    let mut depth: usize = 0;


    loop {

        let hosts = get_group_hosts().len();
        println!("{} ({})", "  ".repeat(depth), hosts)


        for child in get_group_subgroups() {
            inventory_tree(child, depth + 1)
        }
        */

}