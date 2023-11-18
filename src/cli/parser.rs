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
use std::fs;
use std::vec::Vec;
use std::path::PathBuf;
use std::sync::{Arc,RwLock};
use crate::util::io::directory_as_string;
use crate::util::yaml::blend_variables;
use crate::inventory::loading::convert_json_vars;
use crate::util::io::jet_file_open;
use crate::util::yaml::show_yaml_error_in_context;
use crate::cli::version::{GIT_VERSION,GIT_BRANCH,BUILD_TIME};
use std::path::Path;
use std::io;
use std::collections::HashMap;

// the CLI parser struct values hold various values calculated when calling parse() on
// the struct

pub struct CliParser {
    pub playbook_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub inventory_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub role_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub module_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub limit_groups: Vec<String>,
    pub limit_hosts: Vec<String>,
    pub inventory_set: bool,
    pub playbook_set: bool,
    pub mode: u32,
    pub needs_help: bool,
    pub needs_version: bool,
    pub show_hosts: Vec<String>,
    pub show_groups: Vec<String>,
    pub batch_size: Option<usize>,
    pub default_user: String,
    pub sudo: Option<String>,
    pub default_port: i64,
    pub threads: usize,
    pub verbosity: u32,
    pub tags: Option<Vec<String>>,
    pub allow_localhost_delegation: bool,
    pub extra_vars: serde_yaml::Value,
    pub forward_agent: bool,
    pub login_password: Option<String>,
    pub argument_map: HashMap<String, Arguments>,
}

// subcommands are usually required
// FIXME: convert this to an enum

pub const CLI_MODE_UNSET: u32 = 0;
pub const CLI_MODE_SYNTAX: u32 = 1;
pub const CLI_MODE_LOCAL: u32 = 2;
pub const CLI_MODE_CHECK_LOCAL: u32 = 3;
pub const CLI_MODE_SSH: u32 = 4;
pub const CLI_MODE_CHECK_SSH: u32 = 5;
pub const CLI_MODE_SHOW: u32 = 6;
pub const CLI_MODE_SIMULATE: u32 = 7;

fn is_cli_mode_valid(value: &String) -> bool {
    match cli_mode_from_string(value) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn cli_mode_from_string(s: &String) -> Result<u32, String> {
    return match s.as_str() {
        "local"           => Ok(CLI_MODE_LOCAL),
        "check-local"     => Ok(CLI_MODE_CHECK_LOCAL),
        "ssh"             => Ok(CLI_MODE_SSH),
        "check-ssh"       => Ok(CLI_MODE_CHECK_SSH),
        "__simulate"      => Ok(CLI_MODE_SIMULATE),
        "show-inventory"  => Ok(CLI_MODE_SHOW),
        _ => Err(format!("invalid mode: {}", s))
    }
}

// all the supported flags

#[derive(Clone,Debug)]
#[allow(non_camel_case_types)]
pub enum Arguments {
    ARGUMENT_VERSION,
    ARGUMENT_INVENTORY,
    ARGUMENT_INVENTORY_SHORT,
    ARGUMENT_PLAYBOOK,
    ARGUMENT_PLAYBOOK_SHORT,
    ARGUMENT_ROLES,
    ARGUMENT_ROLES_SHORT,
    ARGUMENT_SHOW_GROUPS,
    ARGUMENT_SHOW_HOSTS,
    ARGUMENT_LIMIT_GROUPS,
    ARGUMENT_LIMIT_HOSTS,
    ARGUMENT_HELP,
    ARGUMENT_PORT,
    ARGUMENT_USER,
    ARGUMENT_USER_SHORT,
    ARGUMENT_SUDO,
    ARGUMENT_TAGS,
    ARGUMENT_ALLOW_LOCALHOST,
    ARGUMENT_FORWARD_AGENT,
    ARGUMENT_THREADS,
    ARGUMENT_THREADS_SHORT,
    ARGUMENT_BATCH_SIZE,
    ARGUMENT_VERBOSE,
    ARGUMENT_VERBOSER,
    ARGUMENT_VERBOSEST,
    ARGUMENT_EXTRA_VARS,
    ARGUMENT_EXTRA_VARS_SHORT,
    ARGUMENT_ASK_LOGIN_PASSWORD,
    ARGUMENT_MODULES,
    ARGUMENT_MODULES_SHORT
}

impl Arguments {
    fn as_str(&self) -> &'static str {
        match self {
            Arguments::ARGUMENT_VERSION => "--version",
            Arguments::ARGUMENT_INVENTORY => "--inventory",
            Arguments::ARGUMENT_INVENTORY_SHORT => "-i",
            Arguments::ARGUMENT_PLAYBOOK => "--playbook",
            Arguments::ARGUMENT_PLAYBOOK_SHORT => "-p",
            Arguments::ARGUMENT_ROLES => "--roles",
            Arguments::ARGUMENT_ROLES_SHORT => "-r",
            Arguments::ARGUMENT_MODULES => "--modules",
            Arguments::ARGUMENT_MODULES_SHORT => "-m",
            Arguments::ARGUMENT_SHOW_GROUPS => "--show-groups",
            Arguments::ARGUMENT_SHOW_HOSTS => "--show-hosts",
            Arguments::ARGUMENT_LIMIT_GROUPS => "--limit-groups",
            Arguments::ARGUMENT_LIMIT_HOSTS => "--limit-hosts",
            Arguments::ARGUMENT_HELP => "--help",
            Arguments::ARGUMENT_PORT => "--port",
            Arguments::ARGUMENT_USER => "--user",
            Arguments::ARGUMENT_USER_SHORT => "-u",
            Arguments::ARGUMENT_SUDO => "--sudo",
            Arguments::ARGUMENT_TAGS => "--tags",
            Arguments::ARGUMENT_ALLOW_LOCALHOST => "--allow-localhost-delegation",
            Arguments::ARGUMENT_FORWARD_AGENT => "--forward-agent",
            Arguments::ARGUMENT_THREADS => "--threads",
            Arguments::ARGUMENT_THREADS_SHORT => "-t",
            Arguments::ARGUMENT_BATCH_SIZE => "--batch-size",
            Arguments::ARGUMENT_VERBOSE => "-v",
            Arguments::ARGUMENT_VERBOSER => "-vv",
            Arguments::ARGUMENT_VERBOSEST => "-vvv",
            Arguments::ARGUMENT_EXTRA_VARS => "--extra-vars",
            Arguments::ARGUMENT_EXTRA_VARS_SHORT => "-e",
            Arguments::ARGUMENT_ASK_LOGIN_PASSWORD => "--ask-login-password",
        }
    }
}

fn build_argument_map() -> HashMap<String, Arguments> {
    // this is written backwards mostly for readability
    let inputs = vec![
        (Arguments::ARGUMENT_VERSION, "--version"),
        (Arguments::ARGUMENT_INVENTORY, "--inventory"),
        (Arguments::ARGUMENT_INVENTORY_SHORT, "-i"),
        (Arguments::ARGUMENT_PLAYBOOK, "--playbook"),
        (Arguments::ARGUMENT_PLAYBOOK_SHORT, "-p"),
        (Arguments::ARGUMENT_ROLES, "--roles"),
        (Arguments::ARGUMENT_MODULES, "--modules"),
        (Arguments::ARGUMENT_MODULES_SHORT, "-m"),
        (Arguments::ARGUMENT_ROLES_SHORT, "-r"),
        (Arguments::ARGUMENT_SHOW_GROUPS, "--show-groups"),
        (Arguments::ARGUMENT_SHOW_HOSTS, "--show-hosts"),
        (Arguments::ARGUMENT_LIMIT_GROUPS, "--limit-groups"),
        (Arguments::ARGUMENT_LIMIT_HOSTS, "--limit-hosts"),
        (Arguments::ARGUMENT_HELP, "--help"),
        (Arguments::ARGUMENT_PORT, "--port"),
        (Arguments::ARGUMENT_USER, "--user"),
        (Arguments::ARGUMENT_USER_SHORT, "-u"),
        (Arguments::ARGUMENT_SUDO, "--sudo"),
        (Arguments::ARGUMENT_TAGS, "--tags"),
        (Arguments::ARGUMENT_ALLOW_LOCALHOST, "--allow-localhost-delegation"),
        (Arguments::ARGUMENT_FORWARD_AGENT, "--forward-agent"),
        (Arguments::ARGUMENT_THREADS, "--threads"),
        (Arguments::ARGUMENT_THREADS_SHORT, "-t"),
        (Arguments::ARGUMENT_BATCH_SIZE, "--batch-size"),
        (Arguments::ARGUMENT_VERBOSE, "-v"),
        (Arguments::ARGUMENT_VERBOSER, "-vv"),
        (Arguments::ARGUMENT_VERBOSEST, "-vvv"),
        (Arguments::ARGUMENT_EXTRA_VARS, "--extra-vars"),
        (Arguments::ARGUMENT_EXTRA_VARS_SHORT, "-e"),
        (Arguments::ARGUMENT_ASK_LOGIN_PASSWORD, "--ask-login-password"),
    ];
    let mut map : HashMap<String, Arguments> = HashMap::new();
    for (e,i) in inputs.iter() {
        map.insert(i.to_string(), e.clone());
    }
    return map
}

// output from --version
fn show_version() {
    let header_table = format!("|-|:-\n\
                                |jetp | http://www.jetporch.com/\n\
                                | | (C) Michael DeHaan + contributors, 2023\n\
                                | |\n\
                                | build | {}@{}\n\
                                | | {}\n\
                                | --- | ---\n\
                                | | usage: jetp <MODE> [flags]\n\
                                |-|-", GIT_VERSION, GIT_BRANCH, BUILD_TIME);
    println!("");
    crate::util::terminal::markdown_print(&String::from(header_table));
    println!("");
}

// output from --help

fn show_help() {

    show_version();

    let mode_table = "|:-|:-|:-\n\
                      | *Category* | *Mode* | *Description*\n\
                      | --- | --- | ---\n\
                      | utility: |\n\
                      | | show-inventory | displays inventory, specify --show-groups group1:group2 or --show-hosts host1:host2\n\
                      | |\n\
                      | --- | --- | ---\n\
                      | local machine management: |\n\
                      | | check-local| looks for configuration differences on the local machine\n\
                      | |\n\
                      | | local| manages only the local machine\n\
                      | |\n\
                      | --- | --- | ---\n\
                      | remote machine management: |\n\
                      | | check-ssh | looks for configuration differences over SSH\n\
                      | |\n\
                      | | ssh| manages multiple machines over SSH\n\
                      |-|-";

    crate::util::terminal::markdown_print(&String::from(mode_table));
    println!("");

    let flags_table = "|:-|:-|\n\
                       | *Category* | *Flags* |*Description*\n\
                       | --- | ---\n\
                       | Basics:\n\
                       | | -p, --playbook path1:path2| specifies automation content\n\
                       | |\n\
                       | | -i, --inventory path1:path2| (required for ssh only) specifies which systems to manage\n\
                       | |\n\
                       | | -r, --roles path1:path2| adds additional role search paths. Also uses $JET_ROLES_PATH\n\
                       | |\n\
                       | --- | ---\n\
                       | SSH options:\n\
                       | | --ask-login-password | prompt for the login password on standard input\n\
                       | |\n\
                       | | --batch-size N| fully configure this many hosts before moving to the next batch\n\
                       | |\n\
                       | | --forward-agent | enables SSH agent forwarding but only on specific tasks (ex: git)\n\
                       | |\n\
                       | | --limit-groups group1:group2 | further limits scope for playbook runs\n\
                       | |\n\
                       | | --limit-hosts host1 | further limits scope for playbook runs\n\
                       | |\n\
                       | | --port N | use this default port instead of $JET_SSH_PORT or 22\n\
                       | |\n\
                       | | -t, --threads N| how many parallel threads to use. Alternatively set $JET_THREADS\n\
                       | |\n\
                       | | -u, --user username | use this default username instead of $JET_SSH_USER or $USER\n\
                       | |\n\
                       | --- | ---\n\
                       | Misc options:\n\
                       | | --allow-localhost-delegation | signs off on variable sourcing risks and enables localhost actions with delegate_to\n\
                       | |\n\
                       | | -e, --extra-vars @filename | injects extra variables into the playbook runtime context from a YAML file, or quoted JSON\n\
                       | |\n\
                       | | --sudo username | sudo to this user by default for all tasks\n\
                       | |\n\
                       | | --tags tag1:tag2 | only run tasks or roles with one of these tags\n\
                       | |\n\
                       | | -v -vv -vvv| ever increasing verbosity\n\
                       | |\n\
                       |-|";

    crate::util::terminal::markdown_print(&String::from(flags_table));
    println!("");

}




impl CliParser  {


    // construct a parser with empty result values that will be filled in once parsed.

    pub fn new() -> Self {

        let p = CliParser {
            playbook_paths: Arc::new(RwLock::new(Vec::new())),
            inventory_paths: Arc::new(RwLock::new(Vec::new())),
            role_paths: Arc::new(RwLock::new(Vec::new())),
            module_paths: Arc::new(RwLock::new(Vec::new())),
            needs_help: false,
            needs_version: false,
            mode: CLI_MODE_UNSET,
            show_hosts: Vec::new(),
            show_groups: Vec::new(),
            batch_size: None,
            default_user: match env::var("JET_SSH_USER") {
                Ok(x) => {
                    println!("$JET_SSH_USER: {}", x);
                    x
                },
                Err(_) => match env::var("USER") {
                    Ok(y) => y,
                    Err(_) => String::from("root")
                }
            },
            sudo: None,
            default_port: match env::var("JET_SSH_PORT") {
                Ok(x) => match x.parse::<i64>() {
                    Ok(i)  => {
                        println!("$JET_SSH_PORT: {}", i);
                        i
                    },
                    Err(_) => { println!("environment variable JET_SSH_PORT has an invalid value, ignoring: {}", x); 22 }
                },
                Err(_) => 22
            },
            threads: match env::var("JET_THREADS") {
                Ok(x) => match x.parse::<usize>() {
                        Ok(i)  => i,
                        Err(_) => { println!("environment variable JET_THREADS has an invalid value, ignoring: {}", x); 20 }
                },
                Err(_) => 20
            },
            inventory_set: false,
            playbook_set: false,
            verbosity: 0,
            limit_groups: Vec::new(),
            limit_hosts: Vec::new(),
            tags: None,
            allow_localhost_delegation: false,
            extra_vars: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            forward_agent: false,
            login_password: None,
            argument_map: build_argument_map(),
        };
        return p;
    }

    pub fn show_help(&self) {
        show_help();
    }

    pub fn show_version(&self) {
        show_version();
    }

    // actual CLI parsing happens here

    pub fn parse(&mut self) -> Result<(), String> {

        let mut arg_count: usize = 0;
        let mut next_is_value = false;

        // we go through each CLI arg in a loop, certain arguments take
        // parameters and others do not.

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
                    if argument == Arguments::ARGUMENT_HELP.as_str() {
                        self.needs_help = true;
                        return Ok(())
                    }
                    if argument == Arguments::ARGUMENT_VERSION.as_str() {
                        self.needs_version = true;
                        return Ok(());
                    }

                    // if it's not --help, then the second argument is the
                    // required 'mode' parameter
                    let _result = self.store_mode(argument)?;
                    continue 'each_argument;
                },

                // for the rest of the arguments we need to pay attention to whether
                // we are reading a flag or a value, which alternate
                _ => {

                    if next_is_value == false {

                        // if we expect a flag...
                        // the --help argument requires special handling as it has no
                        // following value
                        if argument_str == Arguments::ARGUMENT_HELP.as_str() {
                            self.needs_help = true;
                            return Ok(())
                        }
                        if argument_str == Arguments::ARGUMENT_VERSION.as_str() {
                            self.needs_version = true;
                            return Ok(())
                        }

                        let mut standalone_arg_found : bool = true;

                        if ! self.argument_map.contains_key(argument_str) {
                            return Err(format!("unrecognized argument: {}", argument_str));
                        } 
                        let arg_enum = self.argument_map.get(argument_str).unwrap().clone();

                        let mut result = match arg_enum {
                            // all parameters that do not take arguments here
                            Arguments::ARGUMENT_ALLOW_LOCALHOST    => self.store_allow_localhost_delegation(),
                            Arguments::ARGUMENT_FORWARD_AGENT      => self.store_forward_agent(),
                            Arguments::ARGUMENT_VERBOSE            => self.increase_verbosity(1),
                            Arguments::ARGUMENT_VERBOSER           => self.increase_verbosity(2),
                            Arguments::ARGUMENT_VERBOSEST          => self.increase_verbosity(3),
                            Arguments::ARGUMENT_ASK_LOGIN_PASSWORD => self.store_login_password(),
                            _ => Ok({ standalone_arg_found = false; next_is_value = true; })
                        };

                        if ! standalone_arg_found {
                            if arg_count == args.len() {
                                return Err(format!("missing argument value for {}", argument_str));    
                            } 
                            else {
                                result = match arg_enum {
                                    // all parameters that do take arguments
                                    Arguments::ARGUMENT_PLAYBOOK          => self.append_playbook(&args[arg_count]),
                                    Arguments::ARGUMENT_PLAYBOOK_SHORT    => self.append_playbook(&args[arg_count]),
                                    Arguments::ARGUMENT_ROLES             => self.append_roles(&args[arg_count]),
                                    Arguments::ARGUMENT_ROLES_SHORT       => self.append_roles(&args[arg_count]),
                                    Arguments::ARGUMENT_MODULES           => self.append_modules(&args[arg_count]),
                                    Arguments::ARGUMENT_MODULES_SHORT     => self.append_modules(&args[arg_count]),
                                    Arguments::ARGUMENT_INVENTORY         => self.append_inventory(&args[arg_count]),
                                    Arguments::ARGUMENT_INVENTORY_SHORT   => self.append_inventory(&args[arg_count]),
                                    Arguments::ARGUMENT_SUDO              => self.store_sudo(&args[arg_count]),
                                    Arguments::ARGUMENT_TAGS              => self.store_tags(&args[arg_count]),
                                    Arguments::ARGUMENT_USER              => self.store_default_user(&args[arg_count]),
                                    Arguments::ARGUMENT_USER_SHORT        => self.store_default_user(&args[arg_count]),
                                    Arguments::ARGUMENT_SHOW_GROUPS       => self.store_show_groups(&args[arg_count]),
                                    Arguments::ARGUMENT_SHOW_HOSTS        => self.store_show_hosts(&args[arg_count]),
                                    Arguments::ARGUMENT_LIMIT_GROUPS      => self.store_limit_groups(&args[arg_count]),
                                    Arguments::ARGUMENT_LIMIT_HOSTS       => self.store_limit_hosts(&args[arg_count]),
                                    Arguments::ARGUMENT_BATCH_SIZE        => self.store_batch_size(&args[arg_count]),
                                    Arguments::ARGUMENT_THREADS           => self.store_threads(&args[arg_count]),
                                    Arguments::ARGUMENT_THREADS_SHORT     => self.store_threads(&args[arg_count]),
                                    Arguments::ARGUMENT_PORT              => self.store_port(&args[arg_count]),
                                    Arguments::ARGUMENT_EXTRA_VARS        => self.store_extra_vars(&args[arg_count]),
                                    Arguments::ARGUMENT_EXTRA_VARS_SHORT  => self.store_extra_vars(&args[arg_count]),
                                    _  => Err(format!("invalid flag: {}", argument_str)),
                                };
                            }
                        }

                        if result.is_err() {
                            return result;
                        }
                        
                    } else {
                        next_is_value = false;
                        continue 'each_argument;
                    }
                } // end argument numbers 3-N
            }


        }

        // make adjustments based on modes
        match self.mode {
            CLI_MODE_LOCAL       => { self.threads = 1 },
            CLI_MODE_CHECK_LOCAL => { self.threads = 1 },
            CLI_MODE_SYNTAX      => { self.threads = 1 },
            CLI_MODE_SHOW        => { self.threads = 1 },
            CLI_MODE_UNSET       => { self.needs_help = true; },
            _ => {}
        }

        if self.playbook_set {
            self.add_role_paths_from_environment()?;
            self.add_implicit_role_paths()?;
            self.add_module_paths_from_environment()?;
            self.add_implicit_module_paths()?;
        }
        Ok(())

    }

    fn store_mode(&mut self, value: &String) -> Result<(), String> {
        if is_cli_mode_valid(value) {
            self.mode = cli_mode_from_string(value).unwrap();
            return Ok(());
        }
        return Err(format!("jetp mode ({}) is not valid, see --help", value))
     }

    fn append_playbook(&mut self, value: &String) -> Result<(), String> {
        self.playbook_set = true;
        match parse_paths(&String::from("-p/--playbook"), value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    if p.is_file() {
                        let full = std::fs::canonicalize(p.as_path()).unwrap();
                        self.playbook_paths.write().unwrap().push(full.to_path_buf()); 
                    } else {
                        return Err(format!("playbook file missing: {:?}", p));
                    }
                }
            },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_PLAYBOOK.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn append_roles(&mut self, value: &String) -> Result<(), String> {

        match parse_paths(&String::from("-r/--roles"), value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    if p.is_dir() {
                        let full = std::fs::canonicalize(p.as_path()).unwrap();
                        self.role_paths.write().unwrap().push(full.to_path_buf()); 
                    } else {
                        return Err(format!("roles directory not found: {:?}", p));
                    }
                }
            },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_ROLES.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn append_modules(&mut self, value: &String) -> Result<(), String> {

        match parse_paths(&String::from("-m/--modules"), value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    if p.is_dir() {
                        let full = std::fs::canonicalize(p.as_path()).unwrap();
                        self.module_paths.write().unwrap().push(full.to_path_buf()); 
                    } else {
                        return Err(format!("modules directory not found: {:?}", p));
                    }
                }
            },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_MODULES.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn append_inventory(&mut self, value: &String) -> Result<(), String> {

        self.inventory_set = true;
        if self.mode == CLI_MODE_LOCAL || self.mode == CLI_MODE_CHECK_LOCAL {
            return Err(format!("--inventory cannot be specified for local modes"));
        }

        match parse_paths(&String::from("-i/--inventory"),value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    self.inventory_paths.write().unwrap().push(p.clone());
                }
            }
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_INVENTORY.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_show_groups(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.show_groups = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_SHOW_GROUPS.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_show_hosts(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.show_hosts = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_SHOW_HOSTS.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_limit_groups(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.limit_groups = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_LIMIT_GROUPS.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_limit_hosts(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.limit_hosts = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_LIMIT_HOSTS.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_tags(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.tags = Some(values); },
            Err(err_msg) =>  return Err(format!("--{} {}", Arguments::ARGUMENT_TAGS.as_str(), err_msg)),
        }
        return Ok(());
    }

    fn store_sudo(&mut self, value: &String) -> Result<(), String> {
        self.sudo = Some(value.clone());
        return Ok(());
    }

    fn store_default_user(&mut self, value: &String) -> Result<(), String> {
        self.default_user = value.clone();
        return Ok(());
    }

    fn store_batch_size(&mut self, value: &String) -> Result<(), String> {
        if self.batch_size.is_some() {
            return Err(format!("{} has been specified already", Arguments::ARGUMENT_BATCH_SIZE.as_str()));
        }
        match value.parse::<usize>() {
            Ok(n) => { self.batch_size = Some(n); return Ok(()); },
            Err(_e) => { return Err(format!("{}: invalid value", Arguments::ARGUMENT_BATCH_SIZE.as_str())); }
        }
    }

    fn store_threads(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<usize>() {
            Ok(n) =>  { self.threads = n; return Ok(()); }
            Err(_e) => { return Err(format!("{}: invalid value", Arguments::ARGUMENT_THREADS.as_str())); }
        }
    }

    fn store_port(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<i64>() {
            Ok(n) =>  { self.default_port = n; return Ok(()); }
            Err(_e) => { return Err(format!("{}: invalid value", Arguments::ARGUMENT_PORT.as_str())); }
        }
    }

    fn store_allow_localhost_delegation(&mut self) -> Result<(), String> {
        self.allow_localhost_delegation = true;
        Ok(())
    }

    fn increase_verbosity(&mut self, amount: u32) -> Result<(), String> {
        self.verbosity = self.verbosity + amount;
        return Ok(())
    }

    fn add_implicit_role_paths(&mut self) -> Result<(), String> {
        let paths = self.playbook_paths.read().unwrap();
        for pb in paths.iter() {
            let dirname = directory_as_string(pb.as_path());
            let mut pathbuf = PathBuf::new();
            pathbuf.push(dirname);
            pathbuf.push("roles");
            if pathbuf.is_dir() {
                let full = fs::canonicalize(pathbuf.as_path()).unwrap();
                self.role_paths.write().unwrap().push(full.to_path_buf());
            } else {
                // ignore as there does not need to be a roles/ dir alongside playbooks
            }
        }
        return Ok(());
    }

    fn add_implicit_module_paths(&mut self) -> Result<(), String> {
        let paths = self.playbook_paths.read().unwrap();
        for pb in paths.iter() {
            let dirname = directory_as_string(pb.as_path());
            let mut pathbuf = PathBuf::new();
            pathbuf.push(dirname);
            pathbuf.push("modules");
            if pathbuf.is_dir() {
                let full = fs::canonicalize(pathbuf.as_path()).unwrap();
                self.module_paths.write().unwrap().push(full.to_path_buf());
            } else {
                // ignore as there does not need to be a modules/ dir alongside playbooks
            }
        }
        return Ok(());
    }

    fn add_role_paths_from_environment(&mut self) -> Result<(), String> {

        let env_roles_path = env::var("JET_ROLES_PATH");
        if env_roles_path.is_ok() {
            match parse_paths(&String::from("$JET_ROLES_PATH"), &env_roles_path.unwrap()) {
                Ok(paths) => {
                    for p in paths.iter() {
                        if p.is_dir() {
                            let full = fs::canonicalize(p.as_path()).unwrap();
                            self.role_paths.write().unwrap().push(full.to_path_buf());
                        }
                    }
                },
                Err(y) => return Err(y)
            };
        }
        return Ok(());
    }

    fn add_module_paths_from_environment(&mut self) -> Result<(), String> {

        let env_modules_path = env::var("JET_MODULES_PATH");
        if env_modules_path.is_ok() {
            match parse_paths(&String::from("$JET_MODULES_PATH"), &env_modules_path.unwrap()) {
                Ok(paths) => {
                    for p in paths.iter() {
                        if p.is_dir() {
                            let full = fs::canonicalize(p.as_path()).unwrap();
                            self.module_paths.write().unwrap().push(full.to_path_buf());
                        }
                    }
                },
                Err(y) => return Err(y)
            };
        }
        return Ok(());
    }

    fn store_extra_vars(&mut self, value: &String) -> Result<(), String> {

        if value.starts_with("@") {
            // input is a filename where the data is YAML

            let rest_of_path = value.replace("@","");
            let path = Path::new(&rest_of_path);
            if ! path.is_file() {
                return Err(format!("--extra-vars parameter with @ expects a file: {}", rest_of_path))
            }
            let extra_file = jet_file_open(path)?;
            let parsed: Result<serde_yaml::Mapping, serde_yaml::Error> = serde_yaml::from_reader(extra_file);
            if parsed.is_err() {
                show_yaml_error_in_context(&parsed.unwrap_err(), &path);
                return Err(format!("edit the file and try again?"));
            }   
            blend_variables(&mut self.extra_vars, serde_yaml::Value::Mapping(parsed.unwrap()));

        } else {
            // input is inline JSON (as YAML wouldn't make sense with the newlines)

            let parsed: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(value);
            let actual = match parsed {
                Ok(x) => x,
                Err(y) => { return Err(format!("inline json is not valid: {}", y)) }
            };   
            let serde_map = convert_json_vars(&actual);
            blend_variables(&mut self.extra_vars, serde_yaml::Value::Mapping(serde_map));
        
        }
        
        return Ok(());

     }

     fn store_forward_agent(&mut self) -> Result<(), String>{
        self.forward_agent = true;
        return Ok(());
     }

     fn store_login_password(&mut self) -> Result<(), String>{
        let mut value = String::new();
        println!("enter login password:");
        match io::stdin().read_line(&mut value) {
            Ok(_) => { self.login_password = Some(String::from(value.trim())); }
            Err(e) =>  return Err(format!("failure reading input: {}", e))
        }
        return Ok(());
     }

}

fn split_string(value: &String) -> Result<Vec<String>, String> {
    return Ok(value.split(":").map(|x| String::from(x)).collect());
}

// accept paths eliminated by ":" and return a list of paths, provided they exist
fn parse_paths(from: &String, value: &String) -> Result<Vec<PathBuf>, String> {
    let string_paths = value.split(":");
    let mut results = Vec::new();
    for string_path in string_paths {
        let mut path_buf = PathBuf::new();
        path_buf.push(string_path);
        if path_buf.exists() {
            results.push(path_buf)
        } else {
            return Err(format!("path ({}) specified by ({}) does not exist", string_path, from));
        }
    }
    return Ok(results);
}
