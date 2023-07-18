pub struct ConnectionCommandResult {
    pub data: String,
    pub exit_status: i32
}

pub trait Connection {

    fn connect(&mut self);  

    // FIXME: add error return objects
    
    fn put_file(&self, data: String, remote_path: String, mode: Option<i32>);

    /*
    fn get_file(&self, remote_path: String) -> String;
    */

    fn run_command(&self, command: String) -> ConnectionCommandResult;


}



//use crate::Ssh;
pub mod ssh;
pub use ssh::Ssh;

