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


use crate::connection::connection::Connection;
use crate::connection::command::CommandResult;
use crate::connection::factory::ConnectionFactory;
use crate::playbooks::context::PlaybookContext;
use crate::connection::local::LocalFactory;
use crate::tasks::*;
use crate::inventory::hosts::Host;
use crate::Inventory;
use std::sync::{Arc,Mutex,RwLock};
use ssh2::Session;
use std::io::{Read,Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use std::net::ToSocketAddrs;

pub struct SshFactory {
    local_factory: LocalFactory,
    localhost: Arc<RwLock<Host>>
    //local_connection: Arc<Mutex<dyn Connection>>,
    //inventory: Arc<RwLock<Inventory>>
}

impl SshFactory { 
    pub fn new(inventory: &Arc<RwLock<Inventory>>) -> Self { 
        Self {
            localhost : inventory.read().expect("inventory read").get_host(&String::from("localhost")),
            local_factory: LocalFactory::new(inventory)
        } 
    }
}

impl ConnectionFactory for SshFactory {
    fn get_connection(&self, context: &Arc<RwLock<PlaybookContext>>, host:&Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>, String> {
        let ctx = context.read().expect("context read");
        let hostname1 = host.read().expect("host read").name.clone();
        if hostname1.eq("localhost") {
            let conn = self.local_factory.get_connection(context, &self.localhost)?;
            return Ok(conn);
        } 

        {
            let cache = ctx.connection_cache.read().unwrap();
            if cache.has_connection(host) {
                let conn = cache.get_connection(host);
                return Ok(conn);
            }
        }

        let (hostname2, user, port) = ctx.get_ssh_connection_details(host);      
        if hostname2.eq("localhost") { 
            let conn = self.local_factory.get_connection(context, &self.localhost)?;
            return Ok(conn); 
        }

        let mut conn = SshConnection::new(Arc::clone(&host), &user, port);
        return match conn.connect() {
            Ok(_)  => { 
                let conn2 : Arc<Mutex<dyn Connection>> = Arc::new(Mutex::new(conn));
                ctx.connection_cache.write().expect("connection cache write").add_connection(&Arc::clone(&host), &Arc::clone(&conn2));
                Ok(conn2)
            },
            Err(x) => { Err(x) } 
        }
    }
}

pub struct SshConnection {
    pub host: Arc<RwLock<Host>>,
    pub username: String,
    pub port: i64,
    pub session: Option<Session>,
}

impl SshConnection {
    pub fn new(host: Arc<RwLock<Host>>, username: &String, port: i64, ) -> Self {
        Self { host: Arc::clone(&host), username: username.clone(), port: port, session: None }
    }
}

impl Connection for SshConnection {

    fn connect(&mut self) -> Result<(), String> {

        if self.session.is_some() {
            return Ok(());
        }

        // derived from docs at https://docs.rs/ssh2/latest/ssh2/
        let session = match Session::new() { Ok(x) => x, Err(_y) => { return Err(String::from("failed to attach to session")); } };
        let mut agent = match session.agent() { Ok(x) => x, Err(_y) => { return Err(String::from("failed to acquire SSH-agent")); } };
        

        // Connect the agent and request a list of identities
        match agent.connect() { Ok(_x) => {}, Err(_y)  => { return Err(String::from("failed to connect to SSH-agent")) }}
        //agent.list_identities().unwrap();
        //for identity in agent.identities().unwrap() {
        //    println!("{}", identity.comment());
        //    let _pubkey = identity.blob();
        //}

 
        // Connect to the local SSH server
        let seconds = Duration::from_secs(10);

        assert!(!self.host.read().expect("host read").name.eq("localhost"));

        let connect_str = format!("{host}:{port}", host=self.host.read().expect("host read").name, port=self.port.to_string());
        

        // connect with timeout requires SocketAddr objects instead of just connection strings
        let addrs_iter = connect_str.as_str().to_socket_addrs();
        
        // check for errors
        let mut addrs_iter2 = match addrs_iter { Err(_x) => { return Err(String::from("unable to resolve")); }, Ok(y) => y };
        let addr = addrs_iter2.next();
        if ! addr.is_some() { return Err(String::from("unable to resolve(2)"));  }
        

        // actually connect here
        let tcp = match TcpStream::connect_timeout(&addr.unwrap(), seconds) { Ok(x) => x, _ => { 
            return Err(format!("SSH connection attempt failed for {}:{}", self.host.read().expect("host read").name, self.port)); } };
        

        // new session & handshake
        let mut sess = match Session::new() { Ok(x) => x, _ => { return Err(String::from("SSH session failed")); } };
        sess.set_tcp_stream(tcp);
        match sess.handshake() { Ok(_) => {}, _ => { return Err(String::from("SSH handshake failed")); } } ;
        

        // try to authenticate with the first identity in the agent.
        match sess.userauth_agent(&self.username) { Ok(_) => {}, _ => { return Err(String::from("SSH userauth_agent failed")); } };
        if !(sess.authenticated()) { return Err("failed to authenticate".to_string()); };
        

        // OS detection
        let uname_result = run_command_low_level(&sess, &String::from("uname -a"));
        match uname_result {
            Ok((_rc,out)) => {
                {
                    match self.host.write().unwrap().set_os_info(&out.clone()) {
                        Ok(_x) => {},
                        Err(_y) => return Err(format!("failed to set OS info"))
                    }
                }
                //match result2 { Ok(_) => {}, Err(s) => { return Err(s.to_string()) } }
            },
            Err((rc,out)) => return Err(format!("uname -a command failed: rc={}, out={}", rc,out))
        }



        // we are connected and the OS was identified ok, so save the session
        self.session = Some(sess);

        return Ok(());
    }

    fn run_command(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let result = run_command_low_level(&self.session.as_ref().unwrap(), cmd);
        match result {
            Ok((rc,s)) => {
                return Ok(handle.command_ok(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: s.clone(), rc: rc }))));
            }, 
            Err((rc,s)) => {
                return Err(handle.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: s.clone(), rc: rc }))));
            }
        }
    }
 
 
    // test pushing a file
    // FIXME: this signature will change -- needs testing
    fn write_data(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, data: &String, remote_path: &String, mode: Option<i32>) -> Result<(),Arc<TaskResponse>> {
        // FIXME: all to the unwrap() calls should be caught
        // FIXME: we should take the mode as input
        let mut real_mode: i32 = 0o644;
        if mode.is_some() {
            real_mode = mode.unwrap();
        }
        let data_size = data.len() as u64;
        let remote_file_result = self.session.as_ref().unwrap().scp_send(
            Path::new(&remote_path), real_mode, data_size, None
        );

        let mut remote_file = match remote_file_result {
            Ok(x) => x,
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed to transmit remote file: {}, {:?}", remote_path, y)))) }
        };
        let bytes = data.as_bytes();
        let write_result = remote_file.write(bytes);
        match write_result {
            Ok(_) => {},
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed to write remote file: {}, {:?}", remote_path, y)))) }
        };
        // Close the channel and wait for the whole content to be transferred
        match remote_file.send_eof() {
            Ok(_) => {},
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed sending eof to remote file: {}, {:?}", remote_path, y)))) }
        };
        match remote_file.wait_eof() {
            Ok(_) => {},
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed waiting for eof on remote file: {}, {:?}", remote_path, y)))) }
        };
        match remote_file.close() {
            Ok(_) => {},
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed closing remote file: {}, {:?}", remote_path, y)))) }
        };
        match remote_file.wait_close() {
            Ok(_) => {},
            Err(y) => { return Err(handle.is_failed(&request, &String::from(format!("failed waiting for file to close on remote file: {}, {:?}", remote_path, y)))) }
        };
        return Ok(());
    }

}

fn run_command_low_level(session: &Session, cmd: &String) -> Result<(i32,String),(i32,String)> {
    // FIXME: catch all these unwraps and return nice errors here

    let mut channel = session.channel_session().unwrap();
    let actual_cmd = format!("{} 2>&1", cmd);

    match channel.exec(&actual_cmd) { Ok(_x) => {}, Err(y) => { return Err((500,y.to_string())) } };
    let mut s = String::new();

    match channel.read_to_string(&mut s) { Ok(_x) => {}, Err(y) => { return Err((500,y.to_string())) } };

    let _w = channel.wait_close();

    let exit_status = match channel.exit_status() { Ok(x) => x, Err(y) => { return Err((500,y.to_string())) } };

    return Ok((exit_status, s.clone()));
}

    // IT WOULD BE NICE TO STREAM!
    // also look at main crate docs for subsystem example?

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



   
   /*
   fn get_file(&self, remote_path: String) -> String {
        String::from("Hey!")

   }
   */




// SHELL may look like this:
/*
channel.shell().unwrap();
for command in commands {
    channel.write_all(command.as_bytes()).unwrap();
    channel.write_all(b"\n").unwrap();
} // Bit inefficient to use separate write calls
channel.send_eof().unwrap();
println!("Waiting for output");
channel.read_to_string(&mut s).unwrap();
println!("{}", s);

https://stackoverflow.com/questions/74512626/how-can-i-run-a-sequence-of-commands-using-ssh2-rs

*/

