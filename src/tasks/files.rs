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

use crate::handle::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use std::sync::Arc;
use serde::Deserialize;
use crate::handle::response::Response;

// this is storage behind all 'and' and 'with' statements in the program, which
// are mostly implemented in task_fsm

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FileAttributesInput {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub mode: Option<String>
}

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FileAttributesEvaluated {
    pub owner: Option<String>,
    pub group: Option<String>,
    pub mode: Option<String>
}

#[derive(Deserialize,Debug,Copy,Clone,PartialEq)]
pub enum Recurse {
    No,
    Yes
}

impl FileAttributesInput {

    // given an octal string, like 0o755 or 755, return the numeric value
    #[inline]
    pub fn is_octal_string(mode: &String) -> bool {
        let octal_no_prefix = str::replace(&mode, "0o", "");
        // this error should be screened out by template() below already but return types are important.
        return match i32::from_str_radix(&octal_no_prefix, 8) {
            Ok(x) => true,
            Err(y) => false 
        }
    }

    // given an octal string, like 0o755 or 755, return the numeric value
    #[inline]
    fn octal_string_to_number(response: &Arc<Response>, request: &Arc<TaskRequest>, mode: &String) -> Result<i32,Arc<TaskResponse>> {
        let octal_no_prefix = str::replace(&mode, "0o", "");
        // this error should be screened out by template() below already but return types are important.
        return match i32::from_str_radix(&octal_no_prefix, 8) {
            Ok(x) => Ok(x),
            Err(y) => { return Err(response.is_failed(&request, &format!("invalid octal value extracted from mode, was {}, {:?}", octal_no_prefix,y))); }
        }
    }

    // template **all** the fields in FileAttributesInput fields, checking values and returning errors as needed
    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<FileAttributesEvaluated>,Arc<TaskResponse>> {

        if input.is_none() {
            return Ok(None);
        }
        
        let input2 = input.as_ref().unwrap();
        let final_mode_value : Option<String>;

        // owner & group is easy but mode is complex
        // makes sure mode is octal and not accidentally enter decimal or hex or leave off the octal prefix
        // as the input field is a YAML string unwanted conversion shouldn't happen but we want to be strict with other tools
        // that might read the file and encourage users to use YAML-spec required input here even though YAML isn't doing
        // the evaluation.

        if input2.mode.is_some()  { 
            let mode_input = input2.mode.as_ref().unwrap();
            let templated_mode_string = handle.template.string(request, &String::from("mode"), &mode_input)?;
            if ! templated_mode_string.starts_with("0o") {
                return Err(handle.response.is_failed(request, &String::from(
                    format!("(a) field (mode) must have an octal-prefixed value of form 0o755, was {}", templated_mode_string)
                )));
            }

            let octal_no_prefix = str::replace(&templated_mode_string, "0o", "");

            // we may have gotten an 0oExampleJunkString which is still not neccessarily valid - so check if it's a number
            // and return the value with the 0o stripped off, for easier use elsewhere
            let decimal_mode = i32::from_str_radix(&octal_no_prefix, 8);
            match decimal_mode {
                Ok(_x) => { 
                    final_mode_value = Some(octal_no_prefix);
                },
                Err(_y) => { 
                    return Err(handle.response.is_failed(request, &String::from(
                        format!("(b) field (mode) must have an octal-prefixed value of form 0o755, was {}", templated_mode_string)
                    )));
                }
            };
        } else {
            // mode was left off in the automation content
            final_mode_value = None;
        }

        return Ok(Some(FileAttributesEvaluated {
            owner:         handle.template.string_option_no_spaces(request, &String::from("owner"), &input2.owner)?,
            group:         handle.template.string_option_no_spaces(request, &String::from("group"), &input2.group)?,
            mode:          final_mode_value,
        }));
    }
}


impl FileAttributesEvaluated {

    // if the action has an evaluated Attributes section, the mode will be stored as an octal string like "777", but we need
    // an integer for some internal APIs like the SSH connection put requests.

    pub fn get_numeric_mode(response: &Arc<Response>, request: &Arc<TaskRequest>, this: &Option<Self>) -> Result<Option<i32>, Arc<TaskResponse>> {

        return match this.is_some() {
            true => {
                let mode = &this.as_ref().unwrap().mode;
                match mode {
                    Some(x) => {
                        let value = FileAttributesInput::octal_string_to_number(response, &request, &x)?;
                        return Ok(Some(value));
                    },
                    None => Ok(None)
                }
            },
            false => Ok(None),
        };
    }

}