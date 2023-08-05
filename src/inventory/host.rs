use std::collections::HashMap;
use serde_yaml::Value;

pub struct Host {
    pub name: String, 
    pub vars: HashMap<String, Value>
}

impl Host  {

    pub fn new(name: String) -> Self {

        Host { 
            name: name,
            vars: HashMap::new(),
        }
    }

}