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

mod cli;
mod inventory;
mod util;
mod playbooks;
mod registry;
mod connection;
mod modules;
mod tasks;
mod handle;

use crate::util::io::{quit};
use crate::inventory::inventory::Inventory;
use crate::inventory::loading::{load_inventory};
use crate::cli::show::{show_inventory_group,show_inventory_host};
use crate::cli::parser::{CliParser};
use crate::cli::playbooks::{playbook_ssh,playbook_local,playbook_check_ssh,playbook_check_local,playbook_simulate}; // FIXME: check modes coming
use std::sync::{Arc,RwLock};
use std::process;

fn main() {
    match liftoff() { Err(e) => quit(&e), _ => {} }
}

fn liftoff() -> Result<(),String> {

    let mut cli_parser = CliParser::new();
    cli_parser.parse()?;

    // jetp --help was given, or no arguments
    if cli_parser.needs_help {
        cli_parser.show_help();
        return Ok(());
    }
    if cli_parser.needs_version {
        cli_parser.show_version();
        return Ok(());
    }

    let inventory : Arc<RwLock<Inventory>> = Arc::new(RwLock::new(Inventory::new()));

    match cli_parser.mode {
        cli::parser::CLI_MODE_SSH | cli::parser::CLI_MODE_CHECK_SSH | cli::parser::CLI_MODE_SHOW | cli::parser::CLI_MODE_SIMULATE => {
            load_inventory(&inventory, Arc::clone(&cli_parser.inventory_paths))?;
            if ! cli_parser.inventory_set {
                return Err(String::from("--inventory is required"));
            }
            if inventory.read().expect("inventory read").hosts.len() == 0 {
                return Err(String::from("no hosts found in --inventory"));
            }
        },
        _ => {
            inventory.write().expect("inventory write").store_host(&String::from("all"), &String::from("localhost"));
        }
    };

    match cli_parser.mode {
        cli::parser::CLI_MODE_SHOW => {},
        _ => {
            if ! cli_parser.playbook_set {
                return Err(String::from("--playbook is required"));
            }
        }
    };

    if cli_parser.threads > 1 {
        rayon::ThreadPoolBuilder::new().num_threads(cli_parser.threads).build_global().expect("build global");
    };

    let exit_status = match cli_parser.mode {
        cli::parser::CLI_MODE_SHOW   => match handle_show(&inventory, &cli_parser) {
            Ok(_) => 0,
            Err(s) => {
                println!("{}", s);
                1
            }
        }
        cli::parser::CLI_MODE_SSH         => playbook_ssh(&inventory, &cli_parser),
        cli::parser::CLI_MODE_CHECK_SSH   => playbook_check_ssh(&inventory, &cli_parser),
        cli::parser::CLI_MODE_LOCAL       => playbook_local(&inventory, &cli_parser),
        cli::parser::CLI_MODE_CHECK_LOCAL => playbook_check_local(&inventory, &cli_parser),
        cli::parser::CLI_MODE_SIMULATE    => playbook_simulate(&inventory, &cli_parser),

        _ => { println!("invalid CLI mode"); 1 }
    };
    if exit_status != 0 {
        process::exit(exit_status);
    }
    return Ok(());
}

pub fn handle_show(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> Result<(), String> {
    // jetp show -i inventory
    // jetp show -i inventory --groups g1:g2
    // jetp show -i inventory --hosts h1:h2
    if parser.show_groups.is_empty() && parser.show_hosts.is_empty() {
        show_inventory_group(inventory, &String::from("all"))?;
    }
    for group_name in parser.show_groups.iter() {
        show_inventory_group(inventory, &group_name.clone())?;
    }
    for host_name in parser.show_hosts.iter() {
        show_inventory_host(inventory, &host_name.clone())?;
    }
    return Ok(());
}

