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

// the guts of CLI command line option parsing
// we don't use any libraries here because they are a bit automagical
// this is clearly the least organized part of jet, sorry

use std::env;
use std::vec::Vec;
use std::path::PathBuf;
use crate::cli::show::{show_inventory_group, show_inventory_host};

// =============================================================================
// PUBLIC API - for main.rs only
// =============================================================================

pub struct CliParser {
    // NEW PARAMETERS?: ADD HERE (AND ELSEWHERE WITH THIS COMMENT)
    pub playbook_paths: Vec<PathBuf>,
    pub inventory_paths: Vec<PathBuf>,
    pub mode: u32,
    pub needs_help: bool,
    pub hosts: Vec<String>,
    pub groups: Vec<String>,
    pub batch_size: Option<u32>,
    // FIXME: threads and other arguments should be added here.
}

pub const CLI_MODE_UNSET: u32 = 0;
pub const CLI_MODE_SYNTAX: u32 = 1;
pub const CLI_MODE_LOCAL: u32 = 2; 
pub const CLI_MODE_CHECK_LOCAL: u32 = 3;
pub const CLI_MODE_SSH: u32 = 4;
pub const CLI_MODE_CHECK_SSH: u32 = 5;
pub const CLI_MODE_SHOW: u32 = 6;

fn is_cli_mode_valid(value: &String) -> bool {
    match cli_mode_from_string(value) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn cli_mode_from_string(s: &String) -> Result<u32, String> {
    return match s.as_str() {
        "local"       => Ok(CLI_MODE_LOCAL),
        "check-local" => Ok(CLI_MODE_CHECK_LOCAL),
        "ssh"         => Ok(CLI_MODE_SSH),
        "check-ssh"   => Ok(CLI_MODE_CHECK_SSH),
        "syntax"      => Ok(CLI_MODE_SYNTAX),
        "show"        => Ok(CLI_MODE_SHOW),
        _ => Err(format!("invalid mode: {}", s))
    }
}

// see also cli_parse.parse and other public methods in main.rs

// =============================================================================
// BEGIN INTERNALS
// =============================================================================

const ARGUMENT_INVENTORY: &'static str = "--inventory";
const ARGUMENT_PLAYBOOK: &'static str  = "--playbook";
const ARGUMENT_GROUPS: &'static str = "--groups";
const ARGUMENT_HOSTS: &'static str = "--hosts";
const ARGUMENT_HELP: &'static str = "--help";

// here's our CLI usage text, it's just hard coded for now
// the markdown isn't too friendly to read but it looks great
fn show_help() {

    let header_table = "|-|:-\n\
                        |jetp | jetporch: the enterprise performance orchestrator |\n\
                        | | (C) Michael DeHaan, 2023\n\
                        | --- | ---\n\
                        | | usage: jetp <MODE> [flags]\n\
                        |-|-";
    
    println!("");
    crate::util::terminal::markdown_print(&String::from(header_table));
    println!("");  

    let mode_table = "|:-|:-|:-|:-:|\n\
                      |  | *Mode* | *Description* | *Makes Changes?* \n\
                      | --- | --- | --- | --- \n\
                      | utility: | | | \n\
                      | | syntax| evaluates input files for errors| no\n\
                      | | | \n\
                      | | simulate| simulates playbook execution | no\n\
                      | | | \n\
                      | | show | displays inventory, playbooks, groups, and hosts | no\n\
                      | | | \n\
                      | --- | --- | --- | --- \n\
                      | local machine management: | | | \n\
                      | | check-local| looks for changes on the local machine| no\n\
                      | | | \n\
                      | | local| manages the local machine| YES\n\
                      | | | \n\
                      | --- | --- | --- | --- \n\
                      | remote machine management: | | | \n\
                      | | check-ssh | connects to nodes over SSH and looks for potential changes| no\n\
                      | | | \n\
                      | | ssh| manages nodes over SSH| YES\n\
                      |-|-|-";

    crate::util::terminal::markdown_print(&String::from(mode_table));
    println!("");

    let flags_table = "|:-|:-|:-|:-:|:-:\n\
                       | |*Flags*|*Description*|*Required For*|*Optional For*|\n\
                       | --- | --- | --- | --- | --- |\n\
                       | basics: | | | |\n\
                       | | --playbook path1:path2| specifies automation content| most | - |\n\
                       | | | | |\n\
                       | | --inventory path1:path2| specifies which systems to manage| most | - |\n\
                       | | | | |\n\
                       | | --roles path1:path2| provides additional role search paths| - | most\n\
                       | | | | |\n\
                       | --- | --- | --- | --- | --- |\n\
                       | scope: | | | |\n\
                       | | --groups group1:group2| for use with playbook narrowing or 'show' | - | most\n\
                       | | | | |\n\
                       | | --hosts host1| for use with playbook narrowing or 'show' | - | most\n\
                       | | | | |\n\
                       | | --tags tag1:tag2| for use with playbook narrowing or 'show' | - | most\n\
                       | | | |\n\
                       | --- | --- | --- | --- | --- |\n\
                       | advanced: | | | |\n\
                       | | --threads tag1:tag2| how many threads to use in SSH operations| - | ssh |\n\
                       | | | | |\n\
                       | | --batch_size tag1:tag2| for canary deployments and rolling updates| - | most |\n\
                       |-|-|-|-|-";

    crate::util::terminal::markdown_print(&String::from(flags_table));
    println!("");
    
}



// ------------------------------------------------------------------------------
// logic behind CLI parsing begins
// we're not using a library like clap as I've been told it changes a lot
// and it would be nice to have flexibility.  There are some capabilities
// missing in the parser but can be added as neede

impl CliParser  {


    // ===========================================================================
    // PUBLIC METHODS
    // ===========================================================================

    // construct a new empty CliParser object, where all the values are empty
    pub fn new() -> Self {

        CliParser { 
            playbook_paths: Vec::new(),
            inventory_paths: Vec::new(),
            needs_help: false,
            mode: CLI_MODE_UNSET,
            hosts: Vec::new(),
            groups: Vec::new(),
            batch_size: None
        }
    }


    // print the usage message
    pub fn show_help(&self) {
        show_help();
    }

    // modiifes the values on the CliParser struct to hold the answers from parsing ARGV
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
                        // add new flags here and elsewhere this comment is found
                        let result = match argument_str {
                            ARGUMENT_PLAYBOOK  => self.store_playbook_value(&args[arg_count]),
                            ARGUMENT_INVENTORY => self.store_inventory_value(&args[arg_count]),
                            ARGUMENT_GROUPS    => self.store_groups_value(&args[arg_count]),
                            ARGUMENT_HOSTS     => self.store_hosts_value(&args[arg_count]),
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


    
    // ===========================================================================
    // PRIVATE METHODS
    // ===========================================================================

    // some arguments may incompatible with each other, so add error handling here
    // add new CLI modes here and elsewhere this comment is found
      
    fn validate_internal_consistency(&mut self) -> Result<(), String> {

        match self.mode {
            CLI_MODE_SSH => (),
            CLI_MODE_CHECK_SSH => (),
            CLI_MODE_LOCAL => (),
            CLI_MODE_CHECK_LOCAL => (),
            CLI_MODE_SYNTAX => (),
            CLI_MODE_SHOW => (),
            CLI_MODE_UNSET => { self.needs_help = true; },
            _ => { panic!("internal error: impossible mode"); }
        } 
        return Ok(())
    }

    // this function is used to store the subcommand in the playbook.  main.rs can
    // perform different logic based on the subcommand, that logic does not happen in this file

    fn store_mode_value(&mut self, value: &String) -> Result<(), String> {
        if is_cli_mode_valid(value) {
            self.mode = cli_mode_from_string(value).unwrap();
            return Ok(());
        }
        return Err(format!("jetp mode ({}) is not valid, see --help", value))
     }
    
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

    fn store_groups_value(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.groups = values; }, 
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_GROUPS, err_msg)),
        }
        return Ok(());
    }

    fn store_hosts_value(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.hosts = values; }, 
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_HOSTS, err_msg)),
        }
        return Ok(());
    }


}

// =====================================================================================
// utility functions
// =====================================================================================

// split a string on colons returning a vector, eventually it may split on a few other
// elements or do some more validation

fn split_string(value: &String) -> Result<Vec<String>, String> {
    return Ok(value.split(":").map(|x| String::from(x)).collect());
}

// accept paths eliminated by ":" and return a list of paths, provided they exist

fn parse_paths(value: &String) -> Result<Vec<PathBuf>, String> {
    
    let string_paths = value.split(":");
    let mut results = Vec::new();

    for string_path in string_paths {

        let mut path_buf = PathBuf::new();
        path_buf.push(string_path);

        if path_buf.exists() {
            results.push(path_buf)
        } else {
            return Err(format!("path ({}) does not exist", string_path));
        }
    }

    return Ok(results);
}

