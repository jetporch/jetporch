// ours
mod connection;
use connection::{Connection};
use connection::ssh::Ssh;

// crates
//use ferris_says::say;
//use ssh2::Session;

// core
//use std::io::prelude::*;
//use std::io::{stdout, BufWriter};
//use std::net::TcpStream;
//use std::path::Path;
//use std::process::Command;

fn main() {
    println!("Hello, world!");

    let mut my_ssh = connection::ssh::Ssh::new(
        "165.227.199.225".to_string(), 
        22, 
        "root".to_string()
    );
    my_ssh.connect();
    let command_result = my_ssh.run_command("ls".to_string());

    println!("command rc: {}", command_result.exit_status);
    println!("command data: {}", command_result.data);
}
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

