use std::collections::HashMap;

pub struct Group {
    pub name: String, 
    pub vars: HashMap<String, serde_yaml::Value>,
    host_names: Vec<String>,
    subgroup_names: Vec<String>,
    //parent_group_names: Vec<String>
}

impl Group  {

    pub fn new(name: String) -> Self {

        Group { 
            name: name,
            vars: HashMap::new(),
            host_names: Vec::new(),
            //parent_group_names: Vec::new(),
            subgroup_names: Vec::new()
        }

    }

    pub fn has_host(&self, host_name: &String) -> bool {
        self.host_names.iter().any(|x| x.eq(host_name))
    }

    pub fn has_subgroup(&self, group_name: &String) -> bool {
        self.subgroup_names.iter().any(|x| x.eq(group_name))
    }
   
    pub fn add_host(&mut self, host_name: String) {
        //let host_name = host_name.clone();
        if !self.has_host(&host_name) {
            self.host_names.push(host_name.clone())
        }
    }

    pub fn add_subgroup(&mut self, group_name: String) {
        //let group_name = group_name.clone();
        if !self.has_subgroup(&group_name) {
            self.subgroup_names.push(group_name.clone());
        }
    }

}