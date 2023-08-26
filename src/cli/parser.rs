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

// we don't use any parsing libraries here because they are a bit too automagical
// this may change later.

use std::env;
use std::vec::Vec;
use std::path::PathBuf;
use std::sync::{Arc,RwLock};

pub struct CliParser {
    pub playbook_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub inventory_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub inventory_set: bool,
    pub playbook_set: bool,
    pub mode: u32,
    pub needs_help: bool,
    pub hosts: Vec<String>,
    pub groups: Vec<String>,
    pub batch_size: Option<usize>,
    pub default_user: Option<String>,
    pub threads: Option<usize>,
    pub verbosity: u32
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

const ARGUMENT_INVENTORY: &'static str = "--inventory";
const ARGUMENT_PLAYBOOK: &'static str  = "--playbook";
const ARGUMENT_GROUPS: &'static str = "--groups";
const ARGUMENT_HOSTS: &'static str = "--hosts";
const ARGUMENT_HELP: &'static str = "--help";
const ARGUMENT_DEFAULT_USER: &'static str = "--default-user";
const ARGUMENT_THREADS: &'static str = "--threads";
const ARGUMENT_BATCH_SIZE: &'static str = "--batch-size";
const ARGUMENT_VERBOSE: &'static str = "-v";

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
                       | | --inventory path1:path2| specifies which systems to manage| ssh | - |\n\
                       | | | | |\n\
                       | | --roles path1:path2| provides additional role search paths| - | most\n\
                       | | | | |\n\
                       | | --default-user username | use this username to connect by default | - | ssh\n\
                       | | | | |\n\
                       | --- | --- | --- | --- | --- |\n\
                       | scope: | | | |\n\
                       | | --groups group1:group2| for use with playbook narrowing or 'show' | - | most\n\
                       | | | | |\n\
                       | | --hosts host1| for use with playbook narrowing or 'show' | - | most\n\
                       | | | | |\n\
                       | --- | --- | --- | --- | --- |\n\
                       | advanced: | | | |\n\
                       | | --threads N| how many threads to use in SSH operations| - | ssh |\n\
                       | | | |\n\
                       |-|-|-|-|-";

    crate::util::terminal::markdown_print(&String::from(flags_table));
    println!("");
    
}


impl CliParser  {

    pub fn new() -> Self {

        CliParser { 
            playbook_paths: Arc::new(RwLock::new(Vec::new())),
            inventory_paths: Arc::new(RwLock::new(Vec::new())),
            needs_help: false,
            mode: CLI_MODE_UNSET,
            hosts: Vec::new(),
            groups: Vec::new(),
            batch_size: None,
            default_user: None,
            threads: None,
            inventory_set: false,
            playbook_set: false,
            verbosity: 0
        }
    }


    pub fn show_help(&self) {
        show_help();
    }

    pub fn parse(&mut self) -> Result<(), String> {
  
        let mut arg_count: usize = 0;
        let mut next_is_value = false;

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
                    let result = self.store_mode_value(argument)?;
                    continue 'each_argument;
                },

                // for the rest of the arguments we need to pay attention to whether
                // we are reading a flag or a value, which alternate
                _ => { 

                    if next_is_value == false {

                        // if we expect a flag...
                        // the --help argument requires special handling as it has no
                        // following value
                        if argument_str == ARGUMENT_HELP {
                            self.needs_help = true;
                            return Ok(())
                        }
                        
                        let result = match argument_str {
                            ARGUMENT_PLAYBOOK     => self.store_playbook_value(&args[arg_count]),
                            ARGUMENT_INVENTORY    => self.store_inventory_value(&args[arg_count]),
                            ARGUMENT_GROUPS       => self.store_groups_value(&args[arg_count]),
                            ARGUMENT_HOSTS        => self.store_hosts_value(&args[arg_count]),
                            ARGUMENT_DEFAULT_USER => self.store_default_user_value(&args[arg_count]),
                            ARGUMENT_BATCH_SIZE   => self.store_batch_size_value(&args[arg_count]),
                            ARGUMENT_THREADS      => self.store_threads_value(&args[arg_count]),
                            ARGUMENT_VERBOSE      => self.increment_verbosity(),

                            _                  => Err(format!("invalid flag: {}", argument_str)),
                            
                        };
                        if result.is_err() { return result; }
                        if ! argument_str.eq(ARGUMENT_VERBOSE) {
                            next_is_value = true;
                        }

                    } else {
                        next_is_value = false;
                        continue 'each_argument;
                    }
                } // end argument numbers 3-N
            }
        } 

        return self.validate_internal_consistency()
    } 
      
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

    fn store_mode_value(&mut self, value: &String) -> Result<(), String> {
        if is_cli_mode_valid(value) {
            self.mode = cli_mode_from_string(value).unwrap();
            return Ok(());
        }
        return Err(format!("jetp mode ({}) is not valid, see --help", value))
     }
    
    fn store_playbook_value(&mut self, value: &String) -> Result<(), String> {
        self.playbook_set = true;
        match parse_paths(value) {
            Ok(paths)  =>  { *self.playbook_paths.write().expect("playbook paths write") = paths; }, 
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_PLAYBOOK, err_msg)),
        }
        return Ok(());
    }

    fn store_inventory_value(&mut self, value: &String) -> Result<(), String> {

        self.inventory_set = true;
        if self.mode == CLI_MODE_LOCAL || self.mode == CLI_MODE_CHECK_LOCAL {
            return Err(format!("--inventory cannot be specified for local modes"));
        }

        match parse_paths(value) {
            Ok(paths)  =>  { *self.inventory_paths.write().expect("inventory paths write") = paths; }, 
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

    fn store_default_user_value(&mut self, value: &String) -> Result<(), String> {
        self.default_user = Some(value.clone());
        return Ok(());
    }

    fn store_batch_size_value(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<usize>() {
            Ok(n) =>  { self.batch_size = Some(n); return Ok(()); },
            Err(e) => { return Err(format!("--{}: invalid value", ARGUMENT_BATCH_SIZE)); }
        }
    }

    fn store_threads_value(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<usize>() {
            Ok(n) =>  { self.threads = Some(n); return Ok(()); }
            Err(e) => { return Err(format!("--{}: invalid value", ARGUMENT_THREADS)); }
        }
    }

    fn increment_verbosity(&mut self) -> Result<(), String> {
        self.verbosity = self.verbosity + 1;
        return Ok(())
    }

}

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

