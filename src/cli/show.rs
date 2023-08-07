use crate::util::terminal::markdown_print;

pub fn show_inventory_tree() -> Result<(),String> {
    println!("IMAGINE A TREE!");
    return Ok(());
}

pub fn show_inventory_host(host_name: String) -> Result<(),String> {

    if !crate::inventory::inventory::has_host(host_name.clone()) {
        return Err(format!("group not found in inventory: {}", host_name));
    }

    let mut buffer = String::new();
    buffer.push_str(&String::from("hey1"));
    markdown_print(&buffer);
    return Ok(());
}

pub fn show_inventory_group(group_name: String) -> Result<(),String> {
    
    println!("*****");
    if !crate::inventory::inventory::has_group(group_name.clone()) {
        return Err(format!("group not found in inventory: {}", group_name));
    }
    
    let mut buffer = String::new();
    buffer.push_str(&String::from(group_name.clone()));
    let groups = crate::inventory::inventory::get_all_group_parents(group_name.clone());
    for parent in groups.iter() {
        println!("parent: {}", parent)
    }
    markdown_print(&buffer);
    return Ok(());

}
