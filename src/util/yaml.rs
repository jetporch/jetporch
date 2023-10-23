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

use std::path::Path;
use std::fs::read_to_string;
use crate::util::terminal::banner;

const YAML_ERROR_SHOW_LINES:usize = 10;
const YAML_ERROR_WIDTH:usize = 180; // things will wrap in terminal anyway

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

pub fn show_yaml_error_in_context(yaml_error: &serde_yaml::Error, path: &Path) {

    println!("");

    let location = yaml_error.location();
    let mut yaml_error_str = String::from(format!("{}", yaml_error));

    yaml_error_str.truncate(YAML_ERROR_WIDTH);
    if yaml_error_str.len() > YAML_ERROR_WIDTH - 3 {
        yaml_error_str.push_str("...");
    }

    if location.is_none() {
        let markdown_table = format!("|:-|\n\
                                      |Error reading YAML file: {}|\n\
                                      |{}|\n\
                                      |-", path.display(), yaml_error_str);
        crate::util::terminal::markdown_print(&markdown_table);
        return;
    }

    // get the line/column info out of the location object
    let location = location.unwrap();
    let error_line = location.line();
    let error_column = location.column();

    let lines: Vec<String> = read_to_string(path).unwrap().lines().map(String::from).collect();
    let line_count = lines.len();

    banner(&format!("Error reading YAML file: {}, {}", path.display(), yaml_error_str).to_string());

    let show_start: usize;
    let mut show_stop : usize = error_line + YAML_ERROR_SHOW_LINES;
    
    if error_line < YAML_ERROR_SHOW_LINES {
        show_start = 0; 
    } else {
        show_start = error_line - YAML_ERROR_SHOW_LINES;
    }

    if show_stop > line_count {
        show_stop = line_count;
    }

    println!("");

    let mut count: usize = 0;

    for line in lines.iter() {
        count = count + 1;
        if count >= show_start && count <= show_stop {
            if count ==  error_line {
                println!("     {count:5}:{error_column:5} | >>> | {}", line);
            } else {
                println!("     {count:5}       |     | {}", line);
            }
        }
    }

    println!("");

}

pub fn blend_variables(a: &mut serde_yaml::Value, b: serde_yaml::Value) {

    match (a, b) {

        (_a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Null) => {
        },

        (a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Mapping(b)) => {
            let a = a.as_mapping_mut().unwrap();
            for (k, v) in b {
                if v.is_sequence() && a.contains_key(&k) && a[&k].is_sequence() {
                    let mut _b = a.get(&k).unwrap().as_sequence().unwrap().to_owned();
                    _b.append(&mut v.as_sequence().unwrap().to_owned());
                    a[&k] = serde_yaml::Value::from(_b);
                    continue;
                }
                if !a.contains_key(&k) {
                    a.insert(k.to_owned(), v.to_owned());
                }
                else {
                    blend_variables(&mut a[&k], v);
                }

            }
        }
        (a, b) => {
            *a = b
        },
    }
}
