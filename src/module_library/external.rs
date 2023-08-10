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

use serde_yaml::Value;
use serde::{Deserialize};
use crate::playbooks::language::AsInteger;
use crate::module_base::common::{IsTask};
use crate::module_base::common::TaskProperties;
use std::collections::HashMap;


crate::module_base::common::define_task!(External { module: String, params: HashMap<String, Value> });
crate::module_base::common::add_task_properties!(External);

impl IsTask for External {

    // FIXME: this is just an example function signature
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}