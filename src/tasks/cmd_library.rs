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
use crate::tasks::FileAttributesInput;

// **IMPORTANT**
// all commands are responsible for screening their inputs within this file
// it is **NOT** permissible to leave this up to the caller. Err on the side
// of over-filtering

pub fn screen_path(path: &String) -> Result<String,String> {
    // NOTE: this only checks paths used in commands and is important because we pass
    // paths to remote commands.
    let path2 = path.trim().to_string();
    let path3 = screen_general_input_strict(&path2)?;
    if path3.find("../").is_some() {
        return Err(format!("climbout paths using ../ are not allowed, ./relative, ~/home, or /absolute are ok: {}", path3));
    }
    return Ok(path3.to_string());
}

// this filtering is applied to all shell arguments in the command library below (if not, it's an error)
// but is automatically also applied to all template calls not marked _unsafe in the evaluate() stages
// of modules. We run everything twice to prevent module coding errors.

pub fn screen_general_input_strict(input: &String) -> Result<String,String> {
    let input2 = input.trim();
    // FIXME: use regex, but compile once, convert to allow list
    // some characters like quotes can break commands but they should not cause operational problems.
    let bad = vec![ ";", "{", "}", "(", ")", "<", ">", "&", "*", "|", "=", "?", "[", "]", "$", "%", "+", "`"];
    for invalid in bad.iter() {
        if input2.find(invalid).is_some() {
            return Err(format!("illegal characters found: {} ('{}')", input2, invalid.to_string()));
        }
    }
    return Ok(input2.to_string());
}

// a slightly lighter version of checking, that allows = signs and such
// this is applied across all commands executed by the system, not just per-parameter checks
// unless run_unsafe is used internally

pub fn screen_general_input_loose(input: &String) -> Result<String,String> {
    let input2 = input.trim();
    // FIXME: use regex, but compile once, convert to allow list
    let bad = vec![ ";", "<", ">", "&", "*", "|", "?", "{", "}", "[", "]", "$", "`"];
    for invalid in bad.iter() {
        if input2.find(invalid).is_some() {
            return Err(format!("illegal characters detected: {} ('{}')", input2, invalid.to_string()));
        }
    }
    return Ok(input2.to_string());
}


pub fn screen_mode(mode: &String) -> Result<String,String> {
    if FileAttributesInput::is_octal_string(&mode) {
        return Ok(mode.clone());
    } else {
        return Err(format!("not an octal string: {}", mode));
    }
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

pub fn get_ownership_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("ls -ld {}", path));
}

pub fn set_owner_command(_os_type: HostOSType, untrusted_path: &String, untrusted_owner: &String) -> Result<String,String> {
    let path = screen_path(untrusted_path)?;
    let owner = screen_general_input_strict(untrusted_owner)?;
    return Ok(format!("chown {} {}", owner, path));
}

pub fn set_group_command(_os_type: HostOSType, untrusted_path: &String, untrusted_group: &String) -> Result<String,String> {
    let path = screen_path(untrusted_path)?;
    let group = screen_general_input_strict(untrusted_group)?;
    return Ok(format!("chgrp {} {}", group, path));
}

pub fn set_mode_command(_os_type: HostOSType, untrusted_path: &String, untrusted_mode: &String) -> Result<String,String> {
    // mode generally does not have to be screened but someone could call a command directly without going through FileAttributes
    // so let's be thorough.
    let path = screen_path(untrusted_path)?;
    let mode = screen_mode(untrusted_mode)?;
    return Ok(format!("chmod {} {}", mode, path));
}



