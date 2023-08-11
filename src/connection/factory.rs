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

use crate::connection::connection::{ConnectionFactory}
use ssh2::Session;
use std::io::{Read,Write};
use std::net::TcpStream;
use std::path::Path;
use crate::connection::ssh::NoConnection;
use crate::connection::local::LocalConnection;
use crate::connection::ssh::SshConnection;
use crate::playbook::traversal::PlaybookContext;

// FIXME: smart connection caching at least for SSH.

trait ConnectionFactory {
    pub fn get_connection(host: String) -> dyn Connection;
}

// ============================================================================================
// NO FACTORY (FOR SYNTAX CHECKS, ETC)
// ============================================================================================

pub struct NoFactory {
}

impl NoFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionFactory for NoFactory  {

    pub get_connection(context: &PlaybookContext, host: String) -> Connection {
        return NoConnection::new();
    }
}

// ============================================================================================
// LOCAL FACTORY
// ============================================================================================

pub struct LocalFactory {
}

impl LocalFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl ConnectionFactory for LocalFactory {
    


    pub get_connection(context: &PlaybookContext, host: String) -> Connection {
        return LocalConnection::new();
    }

}

// ============================================================================================
// SSH FACTORY
// ============================================================================================

pub struct SshFactory {
}

impl SshFactory {
    pub fn new() -> Self {
        Self { }
    }
}

impl ConnectionFactory for SshFactory {
  
    pub get_connection(context: &PlaybookContext, host: String) -> Connection {
        // FIXME: get the remote user from the context
        // FIXME: look for host in blended host variables
        // FIXME: look for port in blended host variables
        // FIXME: return the local connection for hosts named localhost
        return SshConnection::new(host, 22);
    }

}

