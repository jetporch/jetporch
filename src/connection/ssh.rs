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
use crate::handle::response::Response;
use crate::connection::command::Forward;
use crate::connection::local::convert_out;
use std::process::Command;
use std::sync::{Arc,Mutex,RwLock};
use ssh2::Session;
use std::io::{Read,Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;
use std::net::ToSocketAddrs;
use std::fs::File;

// implementation for both Ssh Connections and the Ssh Connection factory

pub struct SshFactory {
    local_factory: LocalFactory,
    localhost: Arc<RwLock<Host>>,
    forward_agent: bool,
    login_password: Option<String>
}

impl SshFactory { 
    pub fn new(inventory: &Arc<RwLock<Inventory>>, forward_agent: bool, login_password: Option<String>) -> Self { 
        // we create a local connection factory for localhost rather than establishing local connections with SSH
        Self {
            localhost : inventory.read().expect("inventory read").get_host(&String::from("localhost")),
            local_factory: LocalFactory::new(inventory),
            forward_agent,
            login_password
        } 
    }
}

impl ConnectionFactory for SshFactory {

    fn get_local_connection(&self, context: &Arc<RwLock<PlaybookContext>>) -> Result<Arc<Mutex<dyn Connection>>, String> {
        return Ok(self.local_factory.get_connection(context, &self.localhost)?);
    }

    fn get_connection(&self, context: &Arc<RwLock<PlaybookContext>>, host:&Arc<RwLock<Host>>) -> Result<Arc<Mutex<dyn Connection>>, String> {
        let ctx = context.read().expect("context read");
        let hostname1 = host.read().expect("host read").name.clone();
        if hostname1.eq("localhost") {
            // if we are asked for a connection to localhost because it's in a group, we'll be called here
            // instead of from get_local_connecton, so have to return the local connection versus assuming SSH
            let conn : Arc<Mutex<dyn Connection>> = self.local_factory.get_connection(context, &self.localhost)?;
            return Ok(conn);
        } 

        {
            // SSH connections are kept open between tasks generally but cleared at many strategic points during playbook traversal
            // between plays, in between batches, etc.
            let cache = ctx.connection_cache.read().unwrap();
            if cache.has_connection(host) {
                let conn = cache.get_connection(host);
                return Ok(conn);
            }
        }

        // how we connect to a host depends on some settings of the play (ssh_port, ssh_user), the CLI (--user) and
        // possibly magic variables on the host.  The context contains all of this logic.
        let (hostname2, user, port, key, passphrase) = ctx.get_ssh_connection_details(host);      
        if hostname2.eq("localhost") { 
            // jet_ssh_hostname was set to localhost, which doesn't make a lot of sense but could happen in testing
            // contrived playbooks when we don't want a lot of real remote hosts
            let conn : Arc<Mutex<dyn Connection>> = self.local_factory.get_connection(context, &self.localhost)?;
            return Ok(conn); 
        }

        // actually connect here
        let mut conn = SshConnection::new(Arc::clone(&host), &user, port, hostname2, self.forward_agent, self.login_password.clone(), key, passphrase);
        return match conn.connect() {
            Ok(_)  => { 
                let conn2 : Arc<Mutex<dyn Connection>> = Arc::new(Mutex::new(conn));
                ctx.connection_cache.write().expect("connection cache write").add_connection(
                    &Arc::clone(&host), &Arc::clone(&conn2));
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
    pub hostname: String, 
    pub session: Option<Session>,
    pub forward_agent: bool,
    pub login_password: Option<String>,
    pub key: Option<String>,
    pub passphrase: Option<String>
}

impl SshConnection {
    pub fn new(host: Arc<RwLock<Host>>, username: &String, port: i64, hostname: String, forward_agent: bool, login_password: Option<String>, key: Option<String>, passphrase: Option<String>) -> Self {
        Self { host: Arc::clone(&host), username: username.clone(), port, hostname, session: None, forward_agent, login_password, key, passphrase }
    }
}

impl Connection for SshConnection {

    fn whoami(&self) -> Result<String,String> {
        // if asked who we are logged in as, it is the user we have connected with
        // sudoers info is on top of that, and this logic is expressed in remote.rs
        return Ok(self.username.clone());
    }

    fn connect(&mut self) -> Result<(), String> {

        if self.session.is_some() {
            // don't re-connect if we are already connected (the code might not try this anyway?)
            return Ok(());
        }

        // derived from docs at https://docs.rs/ssh2/latest/ssh2/
        let session = match Session::new() { Ok(x) => x, Err(_y) => { return Err(String::from("failed to attach to session")); } };
        match session.agent() { 
            Ok(mut agent) => {
                match agent.connect() { 
                    Ok(_) => {}, //x, 
                    Err(_)  => { 
                        println!("Ok, no agent");
                        //return Err(String::from("failed to connect to SSH-agent")) 
                    }
                }
            }, 
            Err(_) => { 
                println!("Ok, no agent 2");
                //return Err(String::from("failed to acquire SSH-agent")); } 
            }
        };

        // Connect the agent
       
        // currently we don't do anything with listing the identities in SSH agent.  It might be helpful to provide a nice error
        // if none were detected

        // Connect to the local SSH server - need to get socketaddrs first in order to use Duration for timeout
        let seconds = Duration::from_secs(10);
        assert!(!self.host.read().expect("host read").name.eq("localhost"));
        let connect_str = format!("{host}:{port}", host=self.hostname, port=self.port.to_string());
        // connect with timeout requires SocketAddr objects instead of just connection strings
        let addrs_iter = connect_str.as_str().to_socket_addrs();
        
        // check for errors
        let mut addrs_iter2 = match addrs_iter { Err(_x) => { return Err(String::from("unable to resolve")); }, Ok(y) => y };
        let addr = addrs_iter2.next();
        if ! addr.is_some() { return Err(String::from("unable to resolve(2)"));  }
        
        // actually connect (finally) here
        let tcp = match TcpStream::connect_timeout(&addr.unwrap(), seconds) { Ok(x) => x, _ => { 
            return Err(format!("SSH connection attempt failed for {}:{}", self.hostname, self.port)); } };
        
        // new session & handshake
        let mut sess = match Session::new() { Ok(x) => x, _ => { return Err(String::from("SSH session failed")); } };
        sess.set_tcp_stream(tcp);
        match sess.handshake() { Ok(_) => {}, _ => { return Err(String::from("SSH handshake failed")); } } ;
        
        //let identities = agent.identities();
        
        if self.login_password.is_some() {
            match sess.userauth_password(&self.username.clone(), self.login_password.clone().unwrap().as_str()) {
                Ok(_) => {},
                Err(x) => {
                    return Err(format!("SSH password authentication failed for user {}: {}", self.username, x));
                }
            }
        }
        if self.key.is_some() {
            let k2 = self.key.as_ref().unwrap().clone();
            let keypath = Path::new(&k2);
            if ! keypath.exists() {
                return Err(format!("cannot find designed keyfile {}", k2));
            }
            match sess.userauth_pubkey_file(&self.username.clone(), None, keypath, self.passphrase.as_deref()) {
                Ok(_) => {},
                Err(x) => {
                    return Err(format!("SSH key authentication failed for user {} with key {:?}: {}", self.username, keypath, x));
                }
            };
        }
        
        if self.key.is_none() && self.login_password.is_none() {
            // no key or password given, try to authenticate with the identities in the agent
            match sess.userauth_agent(&self.username) { 
                Ok(_) => {}, 
                Err(x) => { 
                    return Err(format!("SSH agent authentication failed for user {}: {}", self.username, x));
                }
            };
        }



        if !(sess.authenticated()) { return Err("failed to authenticate".to_string()); };
      
        // OS detection -- always run uname -a on first connect so we know the OS type, which will allow the command library and facts
        // module to work correctly.

        self.session = Some(sess);

        let uname_result = self.run_command_low_level(&String::from("uname -a"));
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


        return Ok(());
    }

    fn run_command(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, cmd: &String, forward: Forward) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let result = match forward {   
            Forward::Yes => match self.forward_agent {
                false => self.run_command_low_level(cmd),
                true  => self.run_command_with_ssh_a(cmd)
            },
            Forward::No => self.run_command_low_level(cmd)
        };

        match result {
            Ok((rc,s)) => {
                // note that non-zero return codes are "ok" to the connection plugin, handle elsewhere!
                return Ok(response.command_ok(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: s.clone(), rc: rc }))));
            }, 
            Err((rc,s)) => {
                return Err(response.command_failed(request, &Arc::new(Some(CommandResult { cmd: cmd.clone(), out: s.clone(), rc: rc }))));
            }
        }
    }

    fn write_data(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, data: &String, remote_path: &String) -> Result<(),Arc<TaskResponse>> {

        // SFTP writing does not allow root to overwrite files root does not own, and does not support sudo. 
        // as such this is a pretty low level write (as is copy_file) and logic around tempfiles and permissions is handled in remote.rs

        // write_data writes a string and is really meant for small files like the template module. Large files should use copy_file instead.

        let session = self.session.as_ref().expect("session not established");
        let sftp_result = session.sftp();
        let sftp = match sftp_result {
            Ok(x) => x,
            Err(y) => { return Err(response.is_failed(request, &format!("sftp connection failed: {y}"))); }
        };
        let sftp_path = Path::new(&remote_path);
        let fh_result = sftp.create(sftp_path);
        let mut fh = match fh_result {
            Ok(x) => x,
            Err(y) => { return Err(response.is_failed(request, &format!("sftp open failed: {y}"))) }
        };
        let bytes = data.as_bytes();
        match fh.write_all(bytes) {
            Ok(_x) => {},
            Err(y) => { return Err(response.is_failed(request, &format!("sftp write failed: {y}"))); }
        }

        return Ok(());
    }

    fn copy_file(&self, response: &Arc<Response>, request: &Arc<TaskRequest>, src: &Path, remote_path: &String) -> Result<(), Arc<TaskResponse>> {

        // this is a streaming copy that should be fine with large files.

        let src_open_result = File::open(src);
        let mut src = match src_open_result {
            Ok(x) => x,
            Err(y) => { return Err(response.is_failed(request, &format!("failed to open source file: {y}"))); }
        };

        let session = self.session.as_ref().expect("session not established");
        let sftp_result = session.sftp();
        let sftp = match sftp_result {
            Ok(x) => x,
            Err(y) => { return Err(response.is_failed(request, &format!("sftp connection failed: {y}"))); }
        };
        let sftp_path = Path::new(&remote_path);
        let fh_result = sftp.create(sftp_path);
        let mut fh = match fh_result {
            Ok(x) => x,
            Err(y) => { return Err(response.is_failed(request, &format!("sftp write failed (1): {y}"))) }
        };

        let chunk_size = 64536;

        loop {
            let mut chunk = Vec::with_capacity(chunk_size);
            let mut taken = std::io::Read::by_ref(&mut src).take(chunk_size as u64);
            let take_result = taken.read_to_end(&mut chunk);
            let n = match take_result {
                Ok(x) => x,
                Err(y) => { return Err(response.is_failed(request, &format!("failed during file transfer: {y}"))); }
            };
            if n == 0 { break; }
            match fh.write(&chunk) {
                Err(y) => { return Err(response.is_failed(request, &format!("sftp write failed: {y}"))); }
                _ => {},

            }
        }
        return Ok(());
    }
}

impl SshConnection {

    fn trim_newlines(&self, s: &mut String) {
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
    }

    fn run_command_low_level(&self, cmd: &String) -> Result<(i32,String),(i32,String)> {
        // FIXME: catch the rare possibility this unwrap fails and return a nice error?
        let session = self.session.as_ref().unwrap();
        let mut channel = match session.channel_session() {
            Ok(x) => x,
            Err(y) => { return Err((500, format!("channel session failed: {:?}", y))); }
        };
        let actual_cmd = format!("{} 2>&1", cmd);
        match channel.exec(&actual_cmd) { Ok(_x) => {}, Err(y) => { return Err((500,y.to_string())) } };
        let mut s = String::new();
        match channel.read_to_string(&mut s) { Ok(_x) => {}, Err(y) => { return Err((500,y.to_string())) } };
        // BOOKMARK: add sudo password prompt (configurable) support here (and below)
        let _w = channel.wait_close();
        let exit_status = match channel.exit_status() { Ok(x) => x, Err(y) => { return Err((500,y.to_string())) } };
        self.trim_newlines(&mut s);
        return Ok((exit_status, s.clone()));
    }

    fn run_command_with_ssh_a(&self, cmd: &String) -> Result<(i32,String),(i32,String)> {
        // this is annoying but libssh2 agent support is not really working, so if we need to SSH -A we need to invoke
        // SSHd directly, which we need to for example with git clones. we will likely use this again
        // for fanout support.

        let mut base = Command::new("ssh");
        let hostname = &self.host.read().unwrap().name;
        let port = format!("{}", self.port);
        let cmd2 = format!("{} 2>&1", cmd);
        let command = base.arg(hostname).arg("-p").arg(port).arg("-l").arg(self.username.clone()).arg("-A").arg(cmd2);
        match command.output() {
            Ok(x) => {
                match x.status.code() {
                    Some(rc) => {
                        let mut out = convert_out(&x.stdout,&x.stderr);
                        self.trim_newlines(&mut out);
                        return Ok((rc, out.clone()))
                    },
                    None => {
                        return Ok((418, String::from("")))
                    }
                }
            },
            Err(_x) => {
                return Err((404, String::from("")))
            }
        };
    }

}
