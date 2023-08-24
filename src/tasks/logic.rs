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
use std::sync::Arc;
use crate::tasks::response::TaskResponse;
use serde::Deserialize;

// this is storage behind all 'and' and 'with' statements in the program, which
// are mostly implemented in task_fsm

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct PreLogicInput {
    pub cond: Option<String>,
    pub sudo: Option<String>
}

#[derive(Debug)]
pub struct PreLogicEvaluated {
    pub cond: Option<String>,
    pub sudo: Option<String>
}

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct PostLogicInput {
    pub changed_when: Option<String>,
    pub delay: Option<String>,
    pub failed_when: Option<String>,
    pub ignore_errors: Option<String>,
    pub save: Option<String>,
    pub retry: Option<String>
}

#[derive(Debug)]
pub struct PostLogicEvaluated {
    pub changed_when: Option<String>,
    pub delay: Option<i64>,
    pub failed_when: Option<String>,
    pub ignore_errors: bool,
    pub save: Option<String>,
    pub retry: Option<i64>
}

impl PreLogicInput {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<PreLogicEvaluated>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(PreLogicEvaluated {
            cond:         input2.cond.clone(), // templated elsewhere!
            sudo:         handle.template_string_option(request, &String::from("sudo"), &input2.sudo)?,
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
            changed_when:  handle.template_string_option(request, &String::from("changed_when"), &input2.changed_when)?,
            delay:         handle.template_integer_option(request, &String::from("delay"), &input2.delay)?,
            failed_when:   handle.template_string_option(request, &String::from("failed_when"), &input2.failed_when)?,
            ignore_errors: handle.template_boolean_option(request, &String::from("ignore_errors"), &input2.ignore_errors)?,
            save:          handle.template_string_option(request, &String::from("save"), &input2.save)?,
            retry:         handle.template_integer_option(request, &String::from("retry"), &input2.retry)?,
        }));
    }
}