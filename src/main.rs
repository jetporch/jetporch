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
mod runner;
mod tasks;

use crate::util::io::{quit};
use crate::inventory::inventory::Inventory;
use crate::inventory::loading::{load_inventory};
use crate::cli::show::{show_inventory_group,show_inventory_host};
use crate::cli::parser::{CliParser};
use crate::cli::playbooks::{playbook_syntax_scan,playbook_ssh,playbook_local}; // FIXME: check modes coming
use std::sync::{Arc,RwLock};
use std::process;
use rayon;

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

    let inventory : Arc<RwLock<Inventory>> = Arc::new(RwLock::new(Inventory::new()));

    match cli_parser.mode {
        cli::parser::CLI_MODE_SSH | cli::parser::CLI_MODE_CHECK_SSH | cli::parser::CLI_MODE_SHOW => {
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

    match cli_parser.threads {
        Some(x) => { rayon::ThreadPoolBuilder::new().num_threads(x).build_global().expect("build global"); }
        None => { rayon::ThreadPoolBuilder::new().num_threads(30).build_global().expect("build global"); }
    };

    let exit_status = match cli_parser.mode {
        cli::parser::CLI_MODE_SHOW   => match handle_show(&inventory, &cli_parser) {
            Ok(_) => 0,
            Err(s) => {
                println!("{}", s);
                1
            }
        }
        cli::parser::CLI_MODE_SYNTAX => playbook_syntax_scan(&inventory, &cli_parser.playbook_paths),
        cli::parser::CLI_MODE_SSH    => playbook_ssh(&inventory, &cli_parser.playbook_paths, cli_parser.default_user),
        cli::parser::CLI_MODE_LOCAL  => playbook_local(&inventory, &cli_parser.playbook_paths),
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
    if parser.groups.is_empty() && parser.hosts.is_empty() {
        return show_inventory_group(inventory, &String::from("all"));
    }
    for group_name in parser.groups.iter() {
        return show_inventory_group(inventory, &group_name.clone());
    }
    for host_name in parser.hosts.iter() {
        return show_inventory_host(inventory, &host_name.clone());
    }
    return Ok(());
}

//******************************************************************************************
// EOF
//******************************************************************************************

// SAVING NOTES ON SSH API FOR NOW

    /*

    let mut my_ssh = connection::ssh::Ssh::new(
        "165.227.199.225".to_string(), 
        22, 
        "root".to_string()
    );
    my_ssh.connect();
    let command_result = my_ssh.run_command("ls".to_string());

    println!("command rc: {}", command_result.exit_status);
    println!("command data: {}", command_result.data);
    */

/*
    // example of calling library

	let stdout = stdout();
    let message = String::from("Hello fellow Rustaceans!");
    let width = message.chars().count();

    let mut writer = BufWriter::new(stdout.lock());
    say(message.as_bytes(), width, &mut writer).unwrap();

    // example of shell call

    let output = Command::new("/bin/cat")
                     .arg("Cargo.toml")
                     .output()
                     .expect("failed to execute process");

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    assert!(output.status.success());

    // Almost all APIs require a `Session` to be available
    println!("ssh time");
    let sess = Session::new().unwrap();
    let mut agent = sess.agent().unwrap();

    // Connect the agent and request a list of identities
    agent.connect().unwrap();
    agent.list_identities().unwrap();

    for identity in agent.identities().unwrap() {
        println!("{}", identity.comment());
        let _pubkey = identity.blob();
    }


   // Connect to the local SSH server
   let tcp = TcpStream::connect("165.227.199.225:22").unwrap();
   let mut sess = Session::new().unwrap();
   sess.set_tcp_stream(tcp);
   sess.handshake().unwrap();

   // Try to authenticate with the first identity in the agent.
   sess.userauth_agent("root").unwrap();
   
   assert!(sess.authenticated());


   let mut channel = sess.channel_session().unwrap();
   channel.exec("ls /tmp").unwrap();
   let mut s = String::new();
   channel.read_to_string(&mut s).unwrap();
   println!("{}", s);
   let _w = channel.wait_close();
   println!("{}", channel.exit_status().unwrap());

   // Make sure we succeeded


   // test pushing a file



    // Write the file
    let mut remote_file = sess.scp_send(Path::new("remote"),
                                    0o644, 10, None).unwrap();
    remote_file.write(b"1234567890").unwrap();

    // Close the channel and wait for the whole content to be transferred
    remote_file.send_eof().unwrap();
    remote_file.wait_eof().unwrap();
    remote_file.close().unwrap();
    remote_file.wait_close().unwrap();


    // pull file


// Connect to the local SSH server
//let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
//let mut sess = Session::new().unwrap();
//sess.set_tcp_stream(tcp);
//sess.handshake().unwrap();
//sess.userauth_agent("username").unwrap();

//let (mut remote_file, stat) = sess.scp_recv(Path::new("remote")).unwrap();
//println!("remote file size: {}", stat.size());
//let mut contents = Vec::new();
//remote_file.read_to_end(&mut contents).unwrap();

// Close the channel and wait for the whole content to be tranferred
//remote_file.send_eof().unwrap();
//remote_file.wait_eof().unwrap();
//remote_file.close().unwrap();
//remote_file.wait_close().unwrap();



}
*/

