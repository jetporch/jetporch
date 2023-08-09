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

// yaml.rs: functions to simplify YAML processing


use std::path::{Path}; // ,PathBuf};
use std::fs::read_to_string;
use crate::util::terminal::{banner};
use serde_yaml::{Mapping};

const YAML_ERROR_SHOW_LINES:usize = 10;
const YAML_ERROR_WIDTH:usize = 180; // things will wrap in terminal anyway

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

pub fn show_yaml_error_in_context(yaml_error: &serde_yaml::Error, path: &Path) {

    println!("");
    // open the YAML file again so we can print it

    // FIXME: may need to trim long error strings as they could contain
    // the whole file (re: yaml_error) inside of format for the error message itself

    // see if there is a YAML line number in the error structure, if not, we can't show the
    // context in the file

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

    // figure out what our start and stop line numbers are when showing
    // where the errors are in the YAML
    let mut show_start: usize = 0;

    // header showing the error, a blank line, then the file contents exerpt
    banner(format!("Error reading YAML file: {}, {}", path.display(), yaml_error_str).to_string());

    if error_line < YAML_ERROR_SHOW_LINES {
        show_start = 1;
    }
    let mut show_stop = error_line + YAML_ERROR_SHOW_LINES;
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

// left takes on values from right
pub fn blend_variables(left_shark: String, right_shark: String) -> String {
    let l = left_shark.clone();
    let r = right_shark.clone();
    let mut left: serde_yaml::Value = serde_yaml::from_str(&l).unwrap();
    let right: serde_yaml::Value = serde_yaml::from_str(&r).unwrap();
    merge_yaml(&mut left, right);
    let yaml_string = &serde_yaml::to_string(&left).unwrap();
    return yaml_string.clone();
}

// ==============================================================================================================
// PRIVATE
// ==============================================================================================================

// adapted from
// https://stackoverflow.com/questions/67727239/how-to-combine-including-nested-array-values-two-serde-yamlvalue-objects

fn merge_yaml(a: &mut serde_yaml::Value, b: serde_yaml::Value) {
 
    /*
    println!("~");
    if a.is_mapping() {
        println!("A: I'm a mapping!");
    } else if a.is_string() {
        println!("A: I'm a string!");
    } else if a.is_null() {
        println!("A: I'm null")
    } else if a.is_sequence() {
        println!("A: I'm sequence");
    } else {
        println!("A: I'm something else!");
    }

    if b.is_mapping() {
        println!("B: I'm a mapping!");
    } else if b.is_string() {
        println!("B: I'm a string!");
    } else if b.is_null() {
        println!("B: I'm null");
    } else if a.is_sequence() {
        println!("B: I'm sequence");
    } else {
        println!("B: I'm something else!");
    }
    */


    match (a, b) {

        (a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Null) => {
        },

        (a @ &mut serde_yaml::Value::Mapping(_), serde_yaml::Value::Mapping(b)) => {
            let a = a.as_mapping_mut().unwrap();
            for (k, v) in b {
                let temp_string = &serde_yaml::to_string(&k).unwrap();
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
                    merge_yaml(&mut a[&k], v); 
                }

            }
        }
        (a, b) => {
            *a = b
        },
    }
}