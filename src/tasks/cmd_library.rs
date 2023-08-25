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

// this is here to prevent typos in module code between Query & Modify 
// match legs. 

use crate::inventory::hosts::HostOSType;

pub fn get_mode_command(os_type: HostOSType, path: &String) -> String {
    return match os_type {
        HostOSType::Linux => String::from(format!("stat --format '%a' {}", path)),
        HostOSType::MacOS => String::from(format!("stat -f '%A' {}", path)),
    }
}
        
