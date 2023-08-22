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

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct CommonLogic {
    pub changed_when: Option<String>,
    pub delay: Option<String>,
    pub register: Option<String>,
    pub retry: Option<String>,
    pub when: Option<String>
}

impl CommonLogic {

    pub fn template(handle: &TaskHandle, request: &Arc<TaskRequest>, input: &Option<Self>) -> Result<Option<CommonLogic>,Arc<TaskResponse>> {
        if input.is_none() {
            return Ok(None);
        }
        let input2 = input.as_ref().unwrap();
        return Ok(Some(Self {
            changed_when: handle.template_option(request, &input2.changed_when)?,
            delay: handle.template_option(request, &input2.delay)?,
            register: handle.template_option(request, &input2.register)?,
            retry: handle.template_option(request, &input2.retry)?,
            when: handle.template_option(request, &input2.when)?,
        }));
    }
}