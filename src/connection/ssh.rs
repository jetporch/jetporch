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

// ===================================================================================
// ABOUT: ssh.rs
// everything about SSH connections.  Note that the factory is programmed to return
// a local (non-SSH) connection for nodes named "localhost", this is *NOT* true
// for nodes named 127.0.0.* so we can still connect to the loopback for testing
// ===================================================================================

use crate::connection::connection::{Connection,ConnectionCommandResult};
use ssh2::Session;
use std::io::{Read,Write};
use std::net::TcpStream;
use std::path::Path;
use crate::connection::factory::ConnectionFactory;
use crate::playbooks::context::PlaybookContext;
use crate::connection::local::LocalConnection;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub struct SshFactory {}

impl SshFactory { 
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionFactory for SshFactory {
    fn get_connection(&self, context: &Arc<RwLock<PlaybookContext>>, host: &String) -> Result<Arc<Mutex<dyn Connection>>, String> {
        if host.eq("localhost") {
            return Ok(Arc::new(Mutex::new(LocalConnection::new())));
        } else {
            let host2 = host.clone();
            let ctx = context.read().unwrap();
            let mut conn = SshConnection::new(
                &host2,
                ctx.get_remote_port(&host2),
                &ctx.get_remote_user(&host2),
            );
            return match conn.connect() {
                Ok(_)  => { Ok(Arc::new(Mutex::new(conn))) },
                Err(x) => { Err(x) } 
            }
        }
    }
}

pub struct SshConnection {
    pub host: String,
    pub port: usize,
    pub username: String,
    pub session: Option<Session>,
}

impl SshConnection {
    pub fn new(host: &String, port: usize, username: &String) -> Self {
        Self { host: host.clone(), port: port, username: username.clone(), session: None }
    }
}

impl Connection for SshConnection {

   fn connect(&mut self) -> Result<(), String> {

       // derived from docs at https://docs.rs/ssh2/latest/ssh2/
    

       // Almost all APIs require a `Session` to be available
       println!("ssh time");
       let session = Session::new().unwrap();

       let mut agent = session.agent().unwrap();
 
        // Connect the agent and request a list of identities
        agent.connect().unwrap();
    
        //agent.list_identities().unwrap();
        //for identity in agent.identities().unwrap() {
        //    println!("{}", identity.comment());
        //    let _pubkey = identity.blob();
        //}
 
 
        // Connect to the local SSH server

        let connect_str = format!("{host}:{port}", host=self.host, port=self.port.to_string());

        let tcp = TcpStream::connect(connect_str).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
 
        // Try to authenticate with the first identity in the agent.
        sess.userauth_agent(&self.username).unwrap();
    
        // FIXME: should return somehow instead and handle it
        if !(sess.authenticated()) {
            return Err("failed to authenticate".to_string());
        }

        self.session = Some(sess);
        return Ok(());

    }


    fn run_command(&self, command: String) -> ConnectionCommandResult {

    
        let mut channel = self.session.as_ref().unwrap().channel_session().unwrap();
        channel.exec(&command).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        //println!("{}", s);
        let _w = channel.wait_close();

        ConnectionCommandResult {
            data: s,
            exit_status: channel.exit_status().unwrap()
        }
        //println!("{}", channel.exit_status().unwrap());
 
    }

    // Make sure we succeeded
 
 
    // test pushing a file
 
    fn put_file(&self, data: String, remote_path: String, mode: Option<i32>) {

        // FIXME: all to the unwrap() calls should be caught

        let mut real_mode: i32 = 0o644;
        if mode.is_some() {
            real_mode = mode.unwrap();
        }
        let data_size = data.len() as u64;


        // Write the file
        let mut remote_file = self.session.as_ref().unwrap().scp_send(
            Path::new(&remote_path),
            real_mode, 
            data_size, 
            None
        ).unwrap();
        remote_file.write(data.as_bytes()).unwrap(); // was b"foo"
 
        // Close the channel and wait for the whole content to be transferred
        remote_file.send_eof().unwrap();
        remote_file.wait_eof().unwrap();
        remote_file.close().unwrap();
        remote_file.wait_close().unwrap();
  
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


}

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