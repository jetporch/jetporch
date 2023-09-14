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
use std::sync::Arc;
use crate::tasks::response::TaskResponse;
use serde::Deserialize;
use std::collections::HashMap;
use crate::handle::template::BlendTarget;

// this is storage behind all 'and' and 'with' statements in the program, which
// are mostly implemented in task_fsm

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct PreLogicInput {
    pub condition: Option<String>,
    pub subscribe: Option<String>,
    pub sudo: Option<String>,
    pub items: Option<ItemsInput>
}

#[derive(Deserialize,Debug)]
#[serde(untagged)]
pub enum ItemsInput {
    ItemsString(String),
    ItemsList(Vec<String>),
    ItemsDict(HashMap<String,serde_yaml::Value>)
}

#[derive(Debug)]
pub struct PreLogicEvaluated {
    pub condition: bool,
    pub subscribe: Option<String>,
    pub sudo: Option<String>,
    pub items: Option<Vec<HashMap<String,serde_yaml::Value>>>
}

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct PostLogicInput {
    pub notify: Option<String>,
    pub ignore_errors: Option<String>,
    pub retry: Option<String>,
    pub delay: Option<String>
}

#[derive(Debug)]
pub struct PostLogicEvaluated {
    pub notify: Option<String>,
    pub ignore_errors: bool,
    pub retry: u64,
    pub delay: u64,
}


impl PreLogicInput {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<PreLogicEvaluated>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(PreLogicEvaluated {
            condition: match &input2.condition {
                Some(cond2) => handle.template.test_condition(request, cond2)?,
                None        => true
            },
            sudo: handle.template.string_option_no_spaces(request, &String::from("sudo"), &input2.sudo)?,
            subscribe: handle.template.no_template_string_option_trim(&input2.subscribe),
            items: template_items(handle, request, &input2.items)?
        }));
    }

}

impl PostLogicInput {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<PostLogicEvaluated>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(PostLogicEvaluated {
            notify: handle.template.string_option_trim(request, &String::from("notify"), &input2.notify)?,
            // unsafe here means the options cannot be sent to the shell, which they are not.
            delay:         handle.template.integer_option(request, &String::from("delay"), &input2.delay, 1)?,
            ignore_errors: handle.template.boolean_option_default_false(request, &String::from("ignore_errors"), &input2.ignore_errors)?,
            retry:         handle.template.integer_option(request, &String::from("retry"), &input2.retry, 0)?,
        }));
    }
}

fn template_items(handle: &TaskHandle, request: &Arc<TaskRequest>, items_input: &Option<ItemsInput>) 
    -> Result<Option<Vec<HashMap<String,serde_yaml::Value>>>, Arc<TaskResponse>> {

    return match items_input {
        None => Ok(None),
        Some(ItemsInput::ItemsString(x)) => {
            let blended = handle.run_state.context.read().unwrap().get_complete_blended_variables(&handle.host, BlendTarget::NotTemplateModule);
            match blended.contains_key(&x) {
                true => {
                    let value : serde_yaml::Value = blended.get(&x).unwrap().clone();
                    Ok(convert_value_to_items_list(&value))
                }, 
                false => {
                    return Err(handle.response.is_failed(request, &format!("variable not found for items: {}", x)))
                }
            }
        }
        Some(ItemsInput::ItemsList(x)) => {
            return Err(handle.response.is_failed(request, &String::from("not supported for with_items yet")))
        }
        Some(ItemsInput::ItemsDict(x)) => {
            return Err(handle.response.is_failed(request, &String::from("not supported for with_items yet")))
        }
    }

}

fn convert_value_to_items_list(value: &serde_yaml::Value) -> Option<Vec<HashMap<String,serde_yaml::Value>>> {
    match value {
        serde_yaml::Value::Sequence(x) => {
            panic!("in progress, got a list");
        }
        serde_yaml::Value::Mapping(x) => {
            panic!("in progress, got a mapping");
        }
        _ => {
            panic!("in progress got something we don't want");
        }
    }
}