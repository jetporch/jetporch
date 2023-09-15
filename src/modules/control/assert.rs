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

use crate::tasks::*;
use crate::handle::handle::TaskHandle;
//#[allow(unused_imports)]
use serde::Deserialize;
use std::sync::Arc;

const MODULE: &'static str = "assert";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct AssertTask {
    pub name: Option<String>,
    pub msg: Option<String>,
    pub r#true: Option<String>,
    pub r#false: Option<String>,
    pub all_true: Option<Vec<String>>,
    pub all_false: Option<Vec<String>>,
    pub some_true: Option<Vec<String>>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

#[allow(dead_code)]
struct AssertAction {
    pub name: String,
    pub msg: Option<String>,
    pub r#true: bool,
    pub r#false: bool,
    pub all_true: Vec<bool>,
    pub all_false: Vec<bool>,
    pub some_true: Vec<bool>

}

impl IsTask for AssertTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(AssertAction {
                    name: self.name.clone().unwrap_or(String::from(MODULE)),
                    msg: handle.template.string_option_unsafe(request, tm, &String::from("msg"), &self.msg)?,
                    r#true: match self.r#true.is_some() {
                            true => handle.template.test_condition(request, tm, &self.r#true.as_ref().unwrap())?,
                            false => true
                    },
                    r#false: match self.r#false.is_some() {
                            true => handle.template.test_condition(request, tm, &self.r#false.as_ref().unwrap())?,
                            false => false
                    },
                    all_true: match self.all_true.is_some() {
                        true => eval_list(handle, request, tm, self.all_true.as_ref().unwrap())?,
                        false => vec![true]
                    },
                    all_false: match self.all_false.is_some() {
                        true => eval_list(handle, request, tm, self.all_false.as_ref().unwrap())?,
                        false => vec![false]
                    },
                    some_true: match self.some_true.is_some() {
                        true => eval_list(handle, request, tm, self.some_true.as_ref().unwrap())?,
                        false => vec![true]
                    }
                }),
                with: Arc::new(PreLogicInput::template(handle, request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, tm, &self.and)?),
            }
        );
    }
}

fn eval_list(handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode, list: &Vec<String>) -> Result<Vec<bool>,Arc<TaskResponse>> {
    let mut results : Vec<bool> = Vec::new();
    for item in list.iter() {
        results.push(handle.template.test_condition(request, tm, item)?);
    }
    return Ok(results);
}

impl IsAction for AssertAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                let mut fail = false;
                if self.r#true == false {
                    println!("XDEBUG: cond1");
                    fail = true;
                }
                else if self.r#false == true {
                    println!("XDEBUG: cond2");
                    fail = true; 
                }
                else if self.all_true.contains(&false) {
                    println!("XDEBUG: cond3");
                    fail = true;
                }
                else if self.all_false.contains(&true) {
                    println!("XDEBUG: cond4");
                    fail = true;
                } 
                else if ! self.some_true.contains(&true) {
                    println!("XDEBUG: cond5");
                    fail = true;
                }
                if fail {
                    if self.msg.is_some() {
                        return Err(handle.response.is_failed(request, &format!("assertion failed: {}", self.msg.as_ref().unwrap())));
                    } else {
                        return Err(handle.response.is_failed(request, &format!("assertion failed")));
                    }
                }
                return Ok(handle.response.is_passive(request));
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }

    }

}