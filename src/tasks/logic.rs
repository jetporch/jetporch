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
use crate::playbooks::templar::TemplateMode;

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

#[derive(Deserialize,Debug,Clone)]
#[serde(untagged)]
pub enum ItemsInput {
    ItemsString(String),
    ItemsList(Vec<String>),
}

#[derive(Debug)]
pub struct PreLogicEvaluated {
    pub condition: bool,
    pub subscribe: Option<String>,
    pub sudo: Option<String>,
    pub items: Option<ItemsInput>
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

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, tm: TemplateMode, input: &Option<Self>) -> Result<Option<PreLogicEvaluated>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(PreLogicEvaluated {
            condition: match &input2.condition {
                Some(cond2) => handle.template.test_condition(request, tm, cond2)?,
                None        => true
            },
            sudo: handle.template.string_option_no_spaces(request, tm, &String::from("sudo"), &input2.sudo)?,
            subscribe: handle.template.no_template_string_option_trim(&input2.subscribe),
            items: input2.items.clone()
        }));
    }

}

impl PostLogicInput {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, tm: TemplateMode, input: &Option<Self>) -> Result<Option<PostLogicEvaluated>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(PostLogicEvaluated {
            notify: handle.template.string_option_trim(request, tm, &String::from("notify"), &input2.notify)?,
            // unsafe here means the options cannot be sent to the shell, which they are not.
            delay:         handle.template.integer_option(request, tm, &String::from("delay"), &input2.delay, 1)?,
            ignore_errors: handle.template.boolean_option_default_false(request, tm, &String::from("ignore_errors"), &input2.ignore_errors)?,
            retry:         handle.template.integer_option(request, tm, &String::from("retry"), &input2.retry, 0)?,
        }));
    }
}

/* this is called from the task_fsm, not above */
pub fn template_items(handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode, items_input: &Option<ItemsInput>) 
    -> Result<Vec<serde_yaml::Value>, Arc<TaskResponse>> {

    return match items_input {

        None => Ok(empty_items_vector()),
        
        // with/items: varname
        Some(ItemsInput::ItemsString(x)) => {
            let blended = handle.run_state.context.read().unwrap().get_complete_blended_variables(
                &handle.host, 
                BlendTarget::NotTemplateModule
            );
            match blended.contains_key(&x) {
                true => {
                    let value : serde_yaml::Value = blended.get(&x).unwrap().clone();
                    match value {
                        serde_yaml::Value::Sequence(vs) => template_serde_sequence(handle, request, tm, vs),
                        _ => {
                            return Err(handle.response.is_failed(request, &format!("with/items variable did not resolve to a list")));
                        }
                    }
                }, 
                false => {
                    return Err(handle.response.is_failed(request, &format!("variable not found for items: {}", x)))
                }
            }
        },
        Some(ItemsInput::ItemsList(x)) => {
            let mut output : Vec<serde_yaml::Value> = Vec::new();
            for item in x.iter() {
                output.push(serde_yaml::Value::String(handle.template.string(request, tm, &String::from("items"), item)?));
            }
            Ok(output)
        }
    }
}

pub fn empty_items_vector() -> Vec<serde_yaml::Value> {
    return vec![serde_yaml::Value::Bool(true)];
}

pub fn template_serde_sequence(
    handle: &TaskHandle, 
    request: &Arc<TaskRequest>, 
    tm: TemplateMode,
    vs: serde_yaml::Sequence) 
    -> Result<Vec<serde_yaml::Value>,Arc<TaskResponse>> {

    let mut output : Vec<serde_yaml::Value> = Vec::new();

    for seq_item in vs.iter() {

        match seq_item {   
            serde_yaml::Value::String(x) => {
                output.push(serde_yaml::Value::String(handle.template.string(request, tm, &String::from("items"), x)?))
            },
            x => { output.push(x.clone()) }
        }
    }
    return Ok(output);
}
