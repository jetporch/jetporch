// this file contains miscellaneous functions to keep lower
// level code (somewhat) out of other classes and to allow us
// to forget how some annoying syntax bits work

// -------------------------------------------------------------------------
// standard imports

use std::fs;
use std::path::{Path}; // ,PathBuf};
use std::fs::ReadDir;
use std::os::unix::fs::PermissionsExt;
use std::process;

// --------------------------------------------------------------------------
// read a directory as per the normal rust way, but map any errors to strings

pub fn jet_read_dir(path: &Path) -> Result<ReadDir, String> {

    // FIXME: this may throw away some of the error details
    // can we include the _x in the format?

    return fs::read_dir(path).map_err(
        |_x| format!("failed to read directory: {}", path.display())
    )

}

// --------------------------------------------------------------------------
// call fn on each path in a subdirectory of the original path, each step is allowed
// to return an error to stop the walking.

pub fn path_walk<F>(path: &Path, mut with_each_path: F) -> Result<(), String> 
   where F: FnMut(&Path) -> Result<(), String> {

    // get the directory result
    let read_result = jet_read_dir(path);

    for entry in read_result.unwrap() {
        // call a function on each entry.  If the function returns an error
        // the function will return that error
        with_each_path(&entry.unwrap().path())?;
    }

    // no errors raised in the callback means the walk was ok.
    Ok(())
}

// --------------------------------------------------------------------------
// open a file per the normal rust way, but map any errors to strings

pub fn jet_file_open(path: &Path) -> Result<std::fs::File, String> {

    // FIXME: this may throw away some of the error details, need to fix
    // can we call format on the error?

    return std::fs::File::open(path).map_err(
        |_x| format!("unable to open file: {}", path.display())
    );
}

// --------------------------------------------------------------------------
// get the last part of the file ignoring the directory part

pub fn path_basename_as_string(path: &Path) -> String {

    // FIXME: what is this Rust mess, seriously? - can we refactor this?
    return path.file_name().unwrap().to_str().unwrap().to_string();

}

// --------------------------------------------------------------------------
// is the path executable?

pub fn is_executable(path: &Path) -> bool {

    // FIXME: (?)the metadata call returns an option -- so this might panic
    // if this is an important error message we should change this
    // to return a result.

    let metadata = fs::metadata(&path).unwrap();
    let permissions = metadata.permissions();

    // FIXME: we're not really seeing if the user can execute this file
    // but just seeing if it is marked executable.  I guess this is ok
    // because the attempt to execute it should fail further down the line.

    if permissions.mode() & 0o111 != 0 {
        return true;
    } else {
        return false;
    }

}

// ----------------------------------------------------------------------------
// quit with a message - please don't use this except in main.rs
// we want to propogate errors all the way up and log/print/etc wherever we can

pub fn quit(s: &String) {
    println!("{}", s); 
    // the exit code is always 1 for now.  May change to add more later.
    process::exit(0x01)
}

