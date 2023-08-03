// --------------------------------------------------------------------------
// standard imports
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

// ---------------------------------------------------------------------------
// external
//use serde_yaml::Value;

// ---------------------------------------------------------------------------
// ours
use crate::inventory::host::Host;

// ----------------------------------------------------------------------------
// inventory groups contain hosts and can contain other subgroups

pub struct Group {
    // all groups have a name
    pub name: String, 

    // a locked vector of locked smart pointers to groups
    pub subgroups: Mutex<Vec<Mutex<Arc<Group>>>>,

    // a locked vector of locked smart pointers to hosts
    pub hosts:     Mutex<Vec<Mutex<Arc<Host>>>>,

    // variables for the group
    pub vars: HashMap<String, serde_yaml::Value>,

    // FIXME: TODO: private flag to keep track of parent chains .... needs to be used.
    pub test: u32 // temporary for debugging

}

//------------------------------------------------------------------------------

impl Group  {

    //--------------------------------------------------------------------------
    // a group is constructed with just a name

    pub fn new(name: String) -> Self {

        Group { 
            name: name,
            subgroups: Mutex::new(Vec::new()),
            hosts: Mutex::new(Vec::new()),
            vars: HashMap::new(),
            test: 42 // temporary for debugging
        }

    }

    //------------------------------------------------------------------------------
    // when we add a host to a group we need a pointer to the host
    
    pub fn add_host(&mut self, host: Arc<Host>) {

        // our hosts collection is wrapped by a mutex that we must unlock
        self.hosts.lock().unwrap().push(
            
            // and each host is a pointer wrapped with a mutex
            Mutex::new(
                Arc::clone(&host)
            )

        );
    }

    //------------------------------------------------------------------------------
    // when we add a subgroup to a group we need a pointer to the group

    pub fn add_subgroup(&mut self, group: Arc<Group>) {

        // our groups collection is wrapped by a mutex that we must unlock
        self.subgroups.lock().unwrap().push(

            // and each group is a pointer wrapped with a mutex
            Mutex::new(
                Arc::clone(&group)
            )

        );
    }

    //------------------------------------------------------------------------------
    // more variable stuff and more to come

}