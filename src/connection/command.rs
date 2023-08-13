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

// ===================================================================================
// ABOUT: commands.rs
// the Command struct wraps commands as executed by // runner::task_handle::TaskHandle 
// to ensure proper usage and status in playbook context. For usage, see many
// modules in modules/
// ===================================================================================

pub struct Command {
    cmd: String,
}

impl Command {

    pub fn new(cmd: String) -> Self {
        Self { 
            cmd: cmd.clone(),
        }
    }

}