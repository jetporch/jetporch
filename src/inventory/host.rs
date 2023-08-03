//------------------------------------------------------------------------------
// std imports

use std::collections::HashMap;

//------------------------------------------------------------------------------
// external stuff

use serde_yaml::Value;

//------------------------------------------------------------------------------
// a host is an addressable system

pub struct Host {
    // all groups have a name
    pub name: String, 

    // variables for the hosts
    pub vars: HashMap<String, Value>
}

//------------------------------------------------------------------------------
// code begins

impl Host  {

    //------------------------------------------------------------------------------
    // a host is constructed with just the hostname and everything else defaults

    pub fn new(name: String) -> Self {

        Host { 
            name: name,
            vars: HashMap::new(),
        }
    }

    //------------------------------------------------------------------------------
    // TODO: method to get all the effective variables in a way that 
    // takes subgroups into account by reversing the relationship

    //------------------------------------------------------------------------------
    // TODO: method to get ssh port based on vars, etc



}