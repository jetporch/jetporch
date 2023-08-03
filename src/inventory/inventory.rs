// =============================================================================================
// core  imports
use std::path::{Path,PathBuf};
use Vec;
use std::sync::Arc;

// =============================================================================================
// external crates
use serde::{Deserialize};

// =============================================================================================
// our stuff

use crate::util::io::{path_walk,jet_file_open,path_basename_as_string,is_executable};
use crate::inventory::group::Group;
use crate::inventory::host::Host;

// =============================================================================================
// inventory is the main struct that holds on to all info about hosts and groups
// it is VERY important.

pub struct Inventory {
    pub groups: Vec<Arc<Group>>,
    pub hosts: Vec<Arc<Host>>
}

// =============================================================================================
// YAML serialization format info
//#[serde(rename_all = "camelCase")]

#[derive(Debug, PartialEq, Deserialize)]
pub struct YamlGroupFileItem {
    host : Option<String>,
    subgroup : Option<String>,
    // FIXME: we'll need this later for other things
    //#[serde(flatten)] f
    //extras: HashMap<String, String>,
}

// =============================================================================================
// begin implementation

impl Inventory  {

    // ------------------------------------------------------------------------------------------
    // construct a new inventory object. This is done in main.rs

    pub fn new() -> Self {

        Inventory { 
            groups: Vec::new(),
            hosts: Vec::new(),
        }
    }

    // ------------------------------------------------------------------------------------------
    // look for the named group in inventory, returning an Option

    pub fn get_group(&self, group_name: String) -> Option<Arc<Group>> {
        return self.groups.iter().find(|x| x.name == group_name).map_or_else(
            || None,
            |found| Some(found.clone())
        )
    }

    // --------------------------------------------------------------------------------------------
    // look for the named host, returning a pointer to it if found
    // if not found, create one and return that as a new object
    // FIXME: some of this arc stuff could benefit from a helper function, perhaps

    pub fn find_or_create_host(&mut self, host_name: String) -> Arc<Host> {

        // grab the hosts collection
        let hosts = &mut self.hosts; 
        
        // find any matching hosts by name, returning an option
        let found = hosts.iter_mut().find(|x| x.name == host_name);

        if found.is_some() {
            // if we found a result, return a copy of the pointer to the object
            return Arc::clone(found.unwrap());
        } else {
            // if no result is found, return a new object
            let ptr = Arc::new(Host::new(host_name.clone()));
            hosts.push(Arc::clone(&ptr));
            return Arc::clone(&ptr);
        }
    }
    // --------------------------------------------------------------------------------------------
    // return a reference to group object if found, if not, make a new one and return that
    // FIXME: some of this arc stuff could benefit from a helper function, perhaps

    pub fn find_or_create_group(&mut self, group_name: String) -> Arc<Group> {

        // grab the groups collection
        let groups = &mut self.groups;   
        
        // find any matching groups by name, returning an option
        let found = groups.iter_mut().find(|x| x.name == group_name);

        if found.is_some() {
            // if we found a result, return a copy of the pointer to the object
            return Arc::clone(found.unwrap());
        } else {
            // if no result is found, return a new object
            let ptr = Arc::new(Group::new(group_name.clone()));
            groups.push(Arc::clone(&ptr));
            return Arc::clone(&ptr);
        }
    }
   
    // ---------------------------------------------------------------------------------------------

    // this function is always called by main.rs to update the inventory 
    // when --inventory is used as a CLI flag on the command line
    // FIXME: it's a bit long as the inventory can be specified a few different ways

    pub fn load_inventory_from_disk(&mut self, inventory_paths: Vec<PathBuf>) -> Result<(), String> {
        
        // for each path passed in on the CLI
        // from arguments like: --inventory path/to/inventory:path/to/other
        for path_buf in inventory_paths {

            // get a path object so we can ask questions of the filesystem
            let path = path_buf.as_path();

            // if the path is a directory
            if path.is_dir() {

                // look for the subdirectory below, such as path/to/inventory/groups
                // this will tell us we have a static inventory
                let groups_pathbuf      = path_buf.join("groups");
                let groups_path         = groups_pathbuf.as_path();
    
                if groups_path.exists() && groups_path.is_dir() {

                    // we have a static inventory since path/to/inventory/groups exists
                    // and fail this function on error
                    println!("XDEBUG: the groups path exists");
                    self.load_non_executable_inventory_structure(true, &path_buf)?;

                } else {

                    // if the groups path didn't exist, we may have been passed a script
                    // via --inventory-path some_inventory.py and need to check if it is 
                    // executable

                    if is_executable(&path) {

                        // first load the variables from the script itself
                        self.load_inventory_from_script(&path_buf)?;

                        // allow other variables to live alongside the script in
                        // ./group_vars and ./host_vars directories
                        // ?
                        // FIXME: need to implement this by looking alongside the file
                        // this method won't do it exaclty without modification?
                        // self.load_non_executable_inventory_structure(false, &path_buf)?;

                    } else {

                        // the file wasn't executable, so someone did something nonsensical
                        // like --inventory-path /path/to/some-regular-file

                        return Err(
                            format!("non-directory path to --inventory ({}) is not executable", 
                            path.display()
                        ))

                    }    
                }
            }
        }
        return Ok(())
    }

    // ---------------------------------------------------------------------------------------------
    // we have identified that an inventory path is a directory
    // if the directory has path/groups as a subdirectory, include_groups will be true, else false
    // we will look for groups/ (potentially), group_vars/, and host_vars directories in the inventory
    // directory, and process all files therein.

    pub fn load_non_executable_inventory_structure(&mut self, include_groups: bool, path_buf: &PathBuf) -> Result<(), String> {
        
        // construct paths for the possible subdirectories of the inventory/ directory
        // FIXME: these strings should be constants eventually to prevent typos
        let group_vars_pathbuf = path_buf.join("group_vars");
        let host_vars_pathbuf  = path_buf.join("host_vars");
        let groups_path        = path_buf.join("groups");
        
        // convert the path buffers back to path objects
        let group_vars_path = group_vars_pathbuf.as_path();
        let host_vars_path = host_vars_pathbuf.as_path();
        
        if include_groups {
            // for non-executable inventory, load the groups/ folder which should be full of YAML
            // files named after groups
            self.load_groups_from_disk(&groups_path)?;
        }
        if group_vars_path.exists() {
            // if there is a group_vars/ directory, attach those variables to the relevant group objects
            self.load_group_vars_from_disk(&group_vars_path)?;
        }
        if host_vars_path.exists() {
            // if there is a host_vars/ directory, attach those variables to the relevant group objects
            self.load_host_vars_from_disk(&host_vars_path)?;
        }
        return Ok(())
    }

    // ---------------------------------------------------------------------------------------------
    // code behind loading groups/ folders

    fn load_groups_from_disk(&mut self, path: &Path) -> Result<(), String> {

        // the input path is something like 'inventory/groups'
        // walk the subdirectories below assuming it is full of YAML files named after groups

        path_walk(path, |subpath| {

            // the basename of the file 'inventory/groups/foo', is 'foo'
            // and is the name of a group
            let group_name = path_basename_as_string(&path).clone();
            println!("XDEBUG: looks like a group: {:?}", group_name);

            // get a handle to the file, on error, let this function fail
            let file = jet_file_open(&path)?;

            // load the YAML file, letting the function fail if there is an error
            self.add_group_file_contents_to_inventory(
                group_name.clone(),
                serde_yaml::from_reader(file).map_or_else(
                    |e| { return Err(format!("yaml parsing failed for file: {}\n{}", path.display(), e)); } ,
                    |x| x 
                )?
            );

            // return ok from the closure
            Ok(())

        });

        // no failures from loading any of the directory items, yay
        Ok(())
    }

    // ---------------------------------------------------------------------------------------------
    // this is a helper function that takes the results of deserializing a YAML groups file
    // and processes the data in the file, to update inventory - making any hosts and groups
    // in that file part of the inventory object as real objects.

    fn add_group_file_contents_to_inventory(&mut self, group_name: String, yaml_entries: &Vec<YamlGroupFileItem>) {
        
        // ensure the referenced group name exists in inventory
        let mut group = self.find_or_create_group(group_name);

        // iterate over each YAML record, which can be either:
        //  - host: hostname
        //  OR
        //  - subgroup: subgroupname
        for item in yaml_entries {
                
            // this entry specifies a host
            if item.host.is_some() {
                // get the optional YAML value
                let hostname = item.host.unwrap();
                // ensure the host datastructure is in inventory
                let host = self.find_or_create_host(hostname);
                // grab a mutable handle on the group object pointer
                let mutable_group = Arc::get_mut(&mut group).unwrap();
                // copy the pointer to the host and add it to the group
                let new_ptr = Arc::clone(&host);
                mutable_group.add_host(new_ptr);
            }
        
            // this entry specifies a subgroup
            if item.subgroup.is_some() {
                // get the optional YAML value
                let subgroupname = item.subgroup.unwrap();
                // ensure the subgroup datastructure is in inventory
                let subgroup = self.find_or_create_group(subgroupname);
                // grab a mutable handle on the group object pointer
                let mutable_group = Arc::get_mut(&mut group).unwrap();
                // copy the pointer to the subgroup and add
                let new_ptr = Arc::clone(&subgroup);
                mutable_group.add_subgroup(new_ptr);
            }
        }

    }
          
            
    /* OLD CODE -- can delete once we are further along
        // ----
        let read_dir = jet_read_dir(&path)?;

        for item in read_dir {

            let path = item.map_err(|_|"read failed")?.path();

            let group_name = path_basename_as_string(&path).to_string();
            println!("looks like a group: {:?}", group_name);

            let file = jet_file_open(&path)?;
            let mut group = self.find_or_create_group(group_name);

            let groups_file_entries: Vec<YamlGroupFileItem> = 
                serde_yaml::from_reader(file).map_or_else(
                    |e| Err(format!("yaml parsing failed for file: {}\n{}", path.display(), e)),
                    |x| x 
                )?;


            
        }
        return Ok(());
    }
    */

    // ---------------------------------------------------------------------------------------------
    // code to load group_vars/ directory
    
    fn load_group_vars_from_disk(&mut self, path: &Path) -> Result<(), String> {
        // FIXME -- walk this path and load each file
        return Err(format!("load_group_vars_from_script_not_implemented: {}", path.display()));
    }

    // ---------------------------------------------------------------------------------------------
    // code to load host_vars/ directory
    
    fn load_host_vars_from_disk(&mut self, path: &Path) -> Result<(), String> {
       // FIXME -- walk this path and load each file
       return Err(format!("load_host_vars_from_script_not_implemented: {}", path.display()));
    }
    
    // ---------------------------------------------------------------------------------------------
    // code to execute inventory script and create from JSON results

    fn load_inventory_from_script(&mut self, path: &Path) -> Result<(), String> {
        // FIXME 
        return Err(format!("load_inventory_from_script_not_implemented: {}", path.display()));
    }


/*

    // (A) it's a directory with a subdirectory called groups, if so, we can we load
    // it and the associated group_vars/ and host_vars/




    // (B) it is an executable file, in which case we can execute it and process
    // the JSON results

    let test_yaml = r#"---
- host: "hostname1"
- host: "hostname2"
- subgroup: "group1"
"#;

    let deser: Vec<GroupNode> = serde_yaml::from_str(&test_yaml)?;
    println!("{:#?}", deser);




    Ok(())


}
*/

}