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
// ABOUT: factory.rs
// the connection factory trait represents logic that can return a connnection for
// a given host record in inventory.  The types of connections per host can be
// heterogenous.
// ===================================================================================

use crate::connection::connection::{Connection};
use crate::playbooks::context::PlaybookContext;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

pub trait ConnectionFactory {
    fn get_connection(&self, context: &Arc<RwLock<PlaybookContext>>, host: &String) -> Result<Arc<Mutex<dyn Connection>>, String>;
}

