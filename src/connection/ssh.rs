use crate::connection::{Connection};

pub struct Ssh {
    pub host: String,
    pub port: u32,
    pub username: String,
}

//impl Ssh {

   /*
   pub fn new(host: String, port: u32, username: String) -> Ssh {
       Ssh {
           host: host,
           port: port,
           username: username
       }
   }
   */

//}

impl Connection for Ssh {

   fn connect(&self) {
       println!("Connected!");
   }

}
