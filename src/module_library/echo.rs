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

#[allow(unused_imports)]
use serde::{Deserialize};

use crate::playbooks::language::AsInteger;
use crate::module_base::common::{IsTask};
use crate::module_base::common::TaskProperties;

crate::module_base::common::define_task!(Echo { path: String });
crate::module_base::common::add_task_properties!(Echo);

impl IsTask for Echo {
    
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }

    fn get_module(&self) -> String {
        return String::from("echo");
    }

}
