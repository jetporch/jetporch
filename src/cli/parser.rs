
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

// ----------------------------------------------------------------------------
// ok so this the guts of the CLI command line parsing.  It's a bit rough
// but it's not really 'critical' code so that's ok for now. It leaves
// options open and I know how it works.  It does some minimal validation
// but leaves most of that for the inventory/playbook loading code and the
// the engine and so on.

// -----------------------------------------------------------------------------
// standard imports
use std::env;
use std::vec::Vec;
use std::path::PathBuf;

// ------------------------------------------------------------------------------
// subcommands for the program are a required argument, this code deals with those
// so in 'jetp ssh' ... and so on, 'ssh' is a cli mode
// ------------------------------------------------------------------------------

pub enum CliMode { 
    // add new CLI modes here and elsewhere this comment is found
    UnsetMode = 0,
    SyntaxOnly = 1,
    Local = 2,
    CheckLocal = 3,
    Ssh = 4,
    CheckSsh = 5,
}

// ------------------------------------------------------------------------------

impl CliMode {

    // ------------------------------------------------------------------------------
    // converts an enum of the valid CLI subcommands into a printable string

    pub fn as_str(&self) -> &'static str {
        // add new CLI modes here and elsewhere this comment is found
        match self {
            CliMode::UnsetMode => "",
            CliMode::SyntaxOnly => "syntax",
            CliMode::Local => "local",
            CliMode::CheckLocal => "check-local",
            CliMode::Ssh => "ssh",
            CliMode::CheckSsh => "check-ssh",
        }
    }
}

// ------------------------------------------------------------------------------
// returns whether the CLI recognizes a given submode, specified as a string

fn is_cli_mode_valid(value: &String) -> bool {
    // add new CLI modes here and elsewhere this comment is found
    let valid_choices = vec!["local", "check-local", "ssh", "check-ssh", "syntax"];
    return valid_choices.contains(&value.as_str());
}

// ------------------------------------------------------------------------------
// given a CLI subcommand string, return the associated enum

fn cli_mode_from_string(s: &String) -> Result<CliMode, String> {
    // add new CLI modes here and elsewhere this comment is found
    return match s.as_str() {
        "local" => Ok(CliMode::Local),
        "check-local" => Ok(CliMode::CheckLocal),
        "ssh"   => Ok(CliMode::Ssh),
        "check-ssh" => Ok(CliMode::CheckSsh),
        "syntax" => Ok(CliMode::SyntaxOnly),
        _ => Err(format!("invalid mode: {}", s))
    }
}

// ------------------------------------------------------------------------------
// here we define constants for all the random CLI flags
// add new CLI modes here and elsewhere this comment is found

const ARGUMENT_INVENTORY: &'static str = "--inventory";
const ARGUMENT_PLAYBOOK: &'static str  = "--playbook";
const ARGUMENT_HELP: &'static str = "--help";

// ------------------------------------------------------------------------------
// here's our CLI usage text, it's just hard coded for now, seems fine

fn show_help() {
    println!("");
    println!("jetp | jetporch : the jet enterprise performance orchetrator");
    println!("(C) Michael DeHaan, 2023");
    println!("------------------------------------------------------------");
    println!("");
    println!("--mode ssh|...");
    println!("--playbook path1:path2");
    println!("--inventory path1:path2");
    println!("");
}

// ------------------------------------------------------------------------------
// storage for CLI results once arguments are loaded, accessed directly
// by main.rs

pub struct CliParser {
    // NEW PARAMETERS?: ADD HERE (AND ELSEWHERE WITH THIS COMMENT)
    pub playbook_paths: Vec<PathBuf>,
    pub inventory_paths: Vec<PathBuf>,
    pub mode: CliMode,
    pub needs_help: bool,
    pub limit_hosts: Option<String>,
    pub limit_groups: Option<String>,
    pub batch_size: Option<u32>,
}

// ------------------------------------------------------------------------------
// logic behind CLI parsing begins
// we're not using a library like clap as I've been told it changes a lot
// and it would be nice to have flexibility.  There are some capabilities
// missing in the parser but can be added as needed


impl CliParser  {


    // ---------------------------------------------------------------------------
    // construct a new empty CliParser object, where all the values are empty

    pub fn new() -> Self {

        CliParser { 
            playbook_paths: Vec::new(),
            inventory_paths: Vec::new(),
            needs_help: false,
            mode: CliMode::UnsetMode,
            limit_hosts: None,
            limit_groups: None,
            batch_size: None
        }
    }

    // ---------------------------------------------------------------------------
    // this is called by main.rs and returns whether it was ok or not, but modifies
    // the values on the CliParser struct to hold the answers from parsing ARGV

    pub fn parse(&mut self) -> Result<(), String> {
  
        // keep track of the argument number
        let mut arg_count: usize = 0;

        // this is a bit of a FSM, deciding if we are parsing a --flag or the
        // the following value
        let mut next_is_value = false;

        // walk the CLI arguments
        let args: Vec<String> = env::args().collect();
        'each_argument: for argument in &args {

            let argument_str = argument.as_str();
            arg_count = arg_count + 1;

            match arg_count {
                // the program name doesn't matter
                1 => continue 'each_argument,

                // the second argument is the subcommand name
                2 => {

                    // we should accept --help anywhere, but this is special
                    // handling as with --help we don't need a subcommand
                    if argument == ARGUMENT_HELP {
                        self.needs_help = true;
                        return Ok(())
                    }
                    
                    // if it's not --help, then the second argument is the 
                    // required 'mode' parameter
                    let result = self.store_mode_value(argument);
                    if result.is_err() {
                        return result;
                    }
                    continue 'each_argument;
                },

                // for the rest of the arguments we need to pay attention to whether
                // we are reading a flag or a value

                _ => { 

                    if next_is_value == false {

                        // if we expect a flag...

                        // the --help argument requires special handling as it has no
                        // following value
                        if argument_str == ARGUMENT_HELP {
                            self.needs_help = true;
                            return Ok(())
                        }
                        
                        // the flag wasn't --help, so see if we have a way to store it
                        // if the user types an invalid --flag, return an error
                        // NEW PARAMETERS?: ADD HERE (AND ELSEWHERE WITH THIS COMMENT)

                        let result = match argument_str {
                            ARGUMENT_PLAYBOOK  => self.store_playbook_value(&args[arg_count]),
                            ARGUMENT_INVENTORY => self.store_inventory_value(&args[arg_count]),
                            _                  => Err(format!("invalid flag: {}", argument_str)),
                            
                        };

                        // if we failed to store the flag value return the result
                        if result.is_err() {
                            return result;
                        }

                        // we need to skip over the next value in ARGV as we've already stored it
                        next_is_value = true;

                    } else {
                        
                        // we've already stored the value by reading ahead when we saw the flag
                        // so the next argument is going to be a --flag
                        next_is_value = false;
                        continue 'each_argument;
                    }
                } // end argument numbers 3-N
            } // end match arg_count
        } // end looping over arguments

        // now that all arguments are loaded in CliParser's struct we need to see if there
        // are any conflicts between them
        return self.validate_internal_consistency()
    } 

    // ---------------------------------------------------------------------------------------
    // print the usage message
    // FIXME: do we need the non-method function?  move code here?

    pub fn show_help(&self) {
        show_help();
    }
    
    // ---------------------------------------------------------------------------------------
    // some arguments may incompatible with each other, so add error handling here
    // FIXME: no implementation yet
    // add new CLI modes here and elsewhere this comment is found
      
    fn validate_internal_consistency(&mut self) -> Result<(), String> {

        match self.mode {
            CliMode::Ssh => (),
            CliMode::CheckSsh => (),
            CliMode::Local => (),
            CliMode::CheckLocal => (),
            CliMode::SyntaxOnly => (),
            CliMode::UnsetMode => { self.needs_help = true; }
        } 
        return Ok(())
    }

    // ---------------------------------------------------------------------------------------
    // this function is used to store the subcommand in the playbook.  main.rs can
    // perform different logic based on the subcommand, that logic does not happen in this file

    fn store_mode_value(&mut self, value: &String) -> Result<(), String> {
        if is_cli_mode_valid(value) {
            self.mode = cli_mode_from_string(value).unwrap();
            return Ok(());
        }
        return Err(format!("jetp mode ({}) is not valid, see --help", value))
     }
    
    // ---------------------------------------------------------------------------------------
    // store --playbook path/to/foo.yml
    // this will raise an error if any of the paths are not present
    // paths can be specified as multiple locations by using a colon between locations

    fn store_playbook_value(&mut self, value: &String) -> Result<(), String> {
        match parse_paths(value) {
            Ok(paths)  =>  { self.playbook_paths = paths; }, 
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_PLAYBOOK, err_msg)),
        }
        return Ok(());
    }
    
    // ---------------------------------------------------------------------------------------
    // store --inventory path/to/inventory/folder and so on
    // this will raise an error if any of the paths are not present
    // paths can be specified as multiple locations by using a colon between locations

    fn store_inventory_value(&mut self, value: &String) -> Result<(), String> {
        match parse_paths(value) {
            Ok(paths)  =>  { self.inventory_paths = paths; }, 
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_INVENTORY, err_msg)),
        }
        return Ok(());
    }

}

// ---------------------------------------------------------------------------------------------
// a helper fucntion to take a string like path1:/path/to/foo/bar and return multiple PathBuf
// objects in a vector.  
// FIXME: if this becomes useful elsewhere, possibly move to utils/io.rs

fn parse_paths(value: &String) -> Result<Vec<PathBuf>, String> {
    
    // get the seperate string tokens for each path as a vector
    let string_paths = value.split(":");
    
    // build a temporary vector to collect the paths that exist
    let mut paths = Vec::new();

    for string_path in string_paths {

        // this is me just cloning the path to make the compiler happy
        // possibly a nicer way to do this
        let mut path = PathBuf::new();
        path.push(string_path);

        if path.exists() {
            // keep track of only the paths that exist which is somewhat 
            // not important due to the error handling below
            paths.push(path)
        } else {
            // if any paths do not exist, fail at the first path
            // we could report all of the paths that don't exist
            // but that doesn't seem important at the moment.
            return Err(format!("path ({}) does not exist", string_path));
        }
    }

    // return all the paths that exist
    // but we will have returned an error already if any did not

    return Ok(paths);
}

