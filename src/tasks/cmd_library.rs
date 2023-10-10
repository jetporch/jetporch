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
use crate::tasks::files::Recurse;

// **IMPORTANT**
//
// all commands are responsible for screening their inputs within this file
// it is **NOT** permissible to leave this up to the caller. Err on the side
// of over-filtering!
//
// most filtering should occur in the module() evaluate code by choosing
// the right template functions.
//
// any argument that allows spaces (such as paths) should be the *last*
// command in any command sequence.

pub fn screen_path(path: &String) -> Result<String,String> {
    // NOTE: this only checks paths used in commands
    let path2 = path.trim().to_string();
    let path3 = screen_general_input_strict(&path2)?;
    return Ok(path3.to_string());
}

// this filtering is applied to all shell arguments in the command library below (if not, it's an error)
// but is automatically also applied to all template calls not marked _unsafe in the evaluate() stages
// of modules. We run everything twice to prevent module coding errors.

pub fn screen_general_input_strict(input: &String) -> Result<String,String> {
    let input2 = input.trim();
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
// unless run_unsafe is used internally. It is assumed that all inputs going into this command
// (parameters) are already sufficiently screened for things that can break shell commands and arguments
// are already quoted.

pub fn screen_general_input_loose(input: &String) -> Result<String,String> {
    let input2 = input.trim();
    let bad = vec![ ";", "<", ">", "&", "*", "?", "{", "}", "[", "]", "$", "`"];
    for invalid in bad.iter() {
        if input2.find(invalid).is_some() {
            return Err(format!("illegal characters detected: {} ('{}')", input2, invalid.to_string()));
        }
    }
    return Ok(input2.to_string());
}

// require that octal inputs be ... octal

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
        // HPUX does not have a stat command
        HostOSType::HPUX  => Ok(format!("perl -e '@x=stat(\"'{}'\"); my $y=sprintf(\"%4o\", $x[2] & 07777); $y=~ s/^\\s+//; print($y);'", path)),
        HostOSType::Linux => Ok(format!("stat --format '%a' '{}'", path)),
        HostOSType::MacOS => Ok(format!("stat -f '%A' '{}'", path)),
        HostOSType::NetBSD => Ok(format!("stat -f '%OLp' '{}'", path)),
        HostOSType::OpenBSD => Ok(format!("stat -f '%OLp' '{}'", path)),
    }
}

pub fn get_sha512_command(os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return match os_type {
        HostOSType::HPUX  => Ok(format!("shasum -a 512 '{}'", path)),
        HostOSType::Linux => Ok(format!("sha512sum '{}'", path)),
        HostOSType::MacOS => Ok(format!("shasum -b -a 512 '{}'", path)),
        HostOSType::NetBSD => Ok(format!("cksum -na sha512 '{}'", path)),
        HostOSType::OpenBSD => Ok(format!("cksum -r -a sha512 '{}'", path)),
    }
}

pub fn get_ownership_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("ls -ld '{}'", path));
}

pub fn get_is_directory_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("ls -ld '{}'", path));
}

pub fn get_touch_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("touch '{}'", path));
}

pub fn get_create_directory_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("mkdir -p '{}'", path));
}

pub fn get_delete_file_command(_os_type: HostOSType, untrusted_path: &String) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    return Ok(format!("rm -f '{}'", path));
}

pub fn get_delete_directory_command(_os_type: HostOSType, untrusted_path: &String, recurse: Recurse) -> Result<String,String>  {
    let path = screen_path(untrusted_path)?;
    match recurse {
        Recurse::No  => { return Ok(format!("rmdir '{}'", path));  },
        Recurse::Yes => { return Ok(format!("rm -rf '{}'", path)); }
    }
}

pub fn set_owner_command(_os_type: HostOSType, untrusted_path: &String, untrusted_owner: &String, recurse: Recurse) -> Result<String,String> {
    let path = screen_path(untrusted_path)?;
    let owner = screen_general_input_strict(untrusted_owner)?;
    match recurse {
        Recurse::No   => { return Ok(format!("chown '{}' '{}'", owner, path));    },
        Recurse::Yes  => { return Ok(format!("chown -R '{}' '{}'", owner, path)); }
    }
}

pub fn set_group_command(_os_type: HostOSType, untrusted_path: &String, untrusted_group: &String, recurse: Recurse) -> Result<String,String> {
    let path = screen_path(untrusted_path)?;
    let group = screen_general_input_strict(untrusted_group)?;
    match recurse {
        Recurse::No   => { return Ok(format!("chgrp '{}' '{}'", group, path));    },
        Recurse::Yes  => { return Ok(format!("chgrp -R '{}' '{}'", group, path)); }
    }
}

pub fn set_mode_command(_os_type: HostOSType, untrusted_path: &String, untrusted_mode: &String, recurse: Recurse) -> Result<String,String> {
    // mode generally does not have to be screened but someone could call a command directly without going through FileAttributes
    // so let's be thorough.
    let path = screen_path(untrusted_path)?;
    let mode = screen_mode(untrusted_mode)?;
    match recurse {
        Recurse::No  => { return Ok(format!("chmod '{}' '{}'", mode, path));    },
        Recurse::Yes => { return Ok(format!("chmod -R '{}' '{}'", mode, path)); }
    }
}



