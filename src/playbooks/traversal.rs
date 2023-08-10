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

use crate::playbooks::language::{Play};
use std::path::PathBuf;

// FIXME: this will take a lot of callback params most likely

pub fn playbook_traversal(playbook_paths: &Vec<PathBuf>) -> Result<(), String> {
// where F, etc

    for path in playbook_paths {

        // let basename = 
        // let dirname = 
        // deserialize playbook 
        // vars in play (other things? other keywords?)
        // walk play
          // roles in play (tasks)
          // if the task is an include, walk that too (function needs to be recursive)
          // roles in play handlers
          // if the handler is an include, walk that too (function needs to be recursive)
        // walk tasks
        // walk handlers
        // walk roles in play (handlers)
        
        println!("traversing a playbook, much to do: {}", path.display());
        /*
        let group_name = path_basename_as_string(&groups_file_path).clone();
        let groups_file = jet_file_open(&groups_file_path)?;
        let groups_file_parse_result: Result<YamlGroup, serde_yaml::Error> = serde_yaml::from_reader(groups_file);
        if groups_file_parse_result.is_err() {
            show_yaml_error_in_context(&groups_file_parse_result.unwrap_err(), &groups_file_path);
            return Err(format!("edit the file and try again?"));
        }   
        let yaml_result = groups_file_parse_result.unwrap();
        add_group_file_contents_to_inventory(
            group_name.clone(), &yaml_result
        );
        */
            
       // Ok(())
    }
    Ok(())
}