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

use std::fs;
use std::path::{Path}; // ,PathBuf};
use std::fs::ReadDir;
use std::os::unix::fs::PermissionsExt;
use std::process;
use std::io::Read;

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

// read a directory as per the normal rust way, but map any errors to strings
pub fn jet_read_dir(path: &Path) -> Result<ReadDir, String> {
    return fs::read_dir(path).map_err(
        |_x| format!("failed to read directory: {}", path.display())
    )
}

// call fn on each path in a subdirectory of the original path, each step is allowed
// to return an error to stop the walking.
pub fn path_walk<F>(path: &Path, mut with_each_path: F) -> Result<(), String> 
   where F: FnMut(&Path) -> Result<(), String> {

    let read_result = jet_read_dir(path);
    for entry in read_result.unwrap() {
        with_each_path(&entry.unwrap().path())?;
    }
    Ok(())
}

// open a file per the normal rust way, but map any errors to strings
pub fn jet_file_open(path: &Path) -> Result<std::fs::File, String> {
    return std::fs::File::open(path).map_err(
        |_x| format!("unable to open file: {}", path.display())
    );
}

pub fn read_local_file(path: &Path) -> Result<String,String> {
    let mut file = jet_file_open(path)?;
    let mut buffer = String::new();
    let read_result = file.read_to_string(&mut buffer);
    match read_result {
        Ok(_) => {},
        Err(x) => {
            return Err(format!("unable to read file: {}, {:?}", path.display(), x));
        }
    };
    return Ok(buffer.clone());
}

// get the last part of the file ignoring the directory part

pub fn path_basename_as_string(path: &Path) -> String {
    return path.file_name().unwrap().to_str().unwrap().to_string();
}

// get the last part of the file ignoring the directory part

pub fn path_as_string(path: &Path) -> String {
    return path.to_str().unwrap().to_string();

}

pub fn directory_as_string(path: &Path) -> String {
    return path.parent().unwrap().to_str().unwrap().to_string();

}

// is the path executable?
pub fn is_executable(path: &Path) -> bool {
    let metadata = fs::metadata(&path).unwrap();
    let permissions = metadata.permissions();
    if permissions.mode() & 0o111 != 0 {
        return true;
    } else {
        return false;
    }
}

// quit with a message - don't use this except in main.rs!

pub fn quit(s: &String) {
    println!("{}", s); 
    process::exit(0x01)
}

