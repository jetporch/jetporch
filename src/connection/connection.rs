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
// ABOUT: connection.rs
// the connection represents an active connection to a node, whether local,
// SSH, or otherwise.  It is fairly low level and modules do not get access to 
// connections directly. Connections are returned by factory::ConnectionFactory trait
// instances.
// ===================================================================================

pub struct ConnectionCommandResult {
    pub data: String,
    pub exit_status: i32
}

pub trait Connection {

    fn connect(&mut self);  

    // FIXME: add error return objects
    
    fn put_file(&self, data: String, remote_path: String, mode: Option<i32>);

    /* 
    FIXME: should add, return result
    fn get_file(&self, remote_path: String) -> String;
    */

    // FIXME should return result
    fn run_command(&self, command: String) -> ConnectionCommandResult;


}