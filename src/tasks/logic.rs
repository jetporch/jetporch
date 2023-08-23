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
pub struct PreLogic {
    pub cond: Option<String>,
    pub sudo: Option<String>
}

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct PostLogic {
    pub changed_when: Option<String>,
    pub delay: Option<i64>,
    pub failed_when: Option<String>,
    pub ignore_errors: Option<bool>,
    pub save: Option<String>,
    pub retry: Option<i64>
}

impl PreLogic {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<PreLogic>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(Self {
            cond: input2.cond.clone(), // templated elsewhere
            sudo: handle.template_option(request, &input2.sudo)?,
        }));
    }
}

impl PostLogic {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<PostLogic>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(Self {
            changed_when: handle.template_option(request, &input2.changed_when)?,
            delay: input2.delay.clone(),
            failed_when: handle.template_option(request, &input2.failed_when)?,
            ignore_errors: input2.ignore_errors.clone(),
            save: handle.template_option(request, &input2.save)?,
            retry: input2.retry.clone(),
        }));
    }
}