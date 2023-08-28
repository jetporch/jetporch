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

// **IMPORTANT**
// all commands are responsible for screening their inputs within this file
// it is **NOT** permissible to leave this up to the caller. Err on the side
// of over-filtering

pub fn screen_path(path: &String) -> Result<String,String> {
    // NOTE: this only checks paths used in commands and is important because we pass
    // paths to remote commands.
    let path2 = path.trim();
    let bad = vec![ ";", ":", "{", "}", "(", ")", "<", ">", "&", "*", "|", "=", "?", "[", "]", "$", "%", "+", "'", "`", " "];
    for invalid in bad.iter() {
        if path.find(invalid).is_some() {
            return Err(format!("invalid characters found in path: {}", path2));
        }
    }
    if path.find("../").is_some() {
        return Err(format!("climbout paths using ../ are not allowed, ./relative, ~/home, or /absolute are ok: {}", path2));
    }
    return Ok(path2.to_string());
}

pub fn get_mode_command(os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return match os_type {
        HostOSType::Linux => Ok(format!("stat --format '%a' {}", path)),
        HostOSType::MacOS => Ok(format!("stat -f '%A' {}", path)),
    }
}
        
pub fn get_sha512_command(os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return match os_type {
        HostOSType::Linux => Ok(format!("sha512sum {}", path)),
        HostOSType::MacOS => Ok(format!("shasum -b -a 512 {}", path)),
    }
}
