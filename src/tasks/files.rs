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

use crate::tasks::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use std::sync::Arc;
use serde::Deserialize;

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

impl FileAttributesInput {

    // given an octal string, like 0o755 or 755, return the numeric value
    #[inline]
    fn octal_string_to_number(handle: &TaskHandle, request: &Arc<TaskRequest>, mode: &String) -> Result<i32,Arc<TaskResponse>> {
        let octal_no_prefix = str::replace(&mode, "0o", "");
        // this error should be screened out by template() below already but return types are important.
        return match i32::from_str_radix(&octal_no_prefix, 8) {
            Ok(x) => Ok(x),
            Err(y) => { return Err(handle.is_failed(&request, &format!("invalid octal value extracted from mode, was {}, {:?}", octal_no_prefix,y))); }
        }
    }

    // template all the fields in attributes, checking values and returning errors as needed
    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<FileAttributesEvaluated>,Arc<TaskResponse>> {
        
        if input.is_none() {
            return Ok(None);
        }
        
        let input2 = input.as_ref().unwrap();
        let mut final_mode_value : Option<String>;

        // makes sure mode is octal and not accidentally decimal or hex
        if input2.mode.is_some()  { 
            let mode_input = input2.mode.as_ref().unwrap();
            let templated_mode_string = handle.template_string(request, &String::from("mode"), &mode_input)?;
            if ! templated_mode_string.starts_with("0o") {
                return Err(handle.is_failed(request, &String::from(
                    format!("(a) field (mode) must have an octal-prefixed value of form 0o755, was {}", templated_mode_string)
                )));
            }

            let octal_no_prefix = str::replace(&templated_mode_string, "0o", "");

            // we may have gotten an 0oJunkString which is still not neccessarily valid - so check if it's a number
            // and return the value with the 0o stripped off, for easier use elsewhere
            let decimal_mode = i32::from_str_radix(&octal_no_prefix, 8);
            match decimal_mode {
                Ok(x) => { 
                    final_mode_value = Some(octal_no_prefix);
                },
                Err(y) => { 
                    return Err(handle.is_failed(request, &String::from(
                        format!("(b) field (mode) must have an octal-prefixed value of form 0o755, was {}", templated_mode_string)
                    )));
                }
            };
        } else {
            final_mode_value = None;
        }

        return Ok(Some(FileAttributesEvaluated {
            owner:         handle.template_string_option(request, &String::from("owner"), &input2.owner)?,
            group:         handle.template_string_option(request, &String::from("group"), &input2.group)?,
            mode:          final_mode_value,
        }));
    }
}


impl FileAttributesEvaluated {

    // if the action has an evaluated Attributes section, the mode will be stored as an octal string like "777", we need
    // an integer for some internal APIs.
    pub fn get_numeric_mode(handle: &TaskHandle, request: &Arc<TaskRequest>, this: &Option<Self>) -> Result<Option<i32>, Arc<TaskResponse>> {
        
        return match this.is_some() {
            true => {
                let mode = &this.as_ref().unwrap().mode;
                match mode {
                    Some(x) => {
                        let value = FileAttributesInput::octal_string_to_number(&handle, &request, &x)?;
                        return Ok(Some(value));
                    },
                    None => Ok(None)
                }
            },
            false => Ok(None),
        };
    }

}