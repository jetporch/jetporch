// ours
use crate::connection::*;

// external crates
use ssh2::Session;

// stdlib
//use std::io::prelude::*;
//use std::io::{stdout, BufWriter};
use std::io::{Read,Write};
use std::net::TcpStream;
use std::path::Path;
//use std::process::Command;
//use std::fmt::{format};


pub struct Ssh {
    pub host: String,
    pub port: u32,
    pub username: String,
    pub session: Option<Session>,
}

impl Ssh {
    pub fn new(host: String, port: u32, username: String) -> Self {
        Self { host: host, port: port, username: username, session: None }
    }
}

impl Connection for Ssh {

   fn connect(&mut self) {

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
    
        assert!(sess.authenticated());

        self.session = Some(sess);

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
