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

use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use crate::tasks::handle::TaskHandle;
use std::sync::Arc;
use std::marker::{Send,Sync};

pub trait Connection : Send + Sync {

    fn connect(&mut self) -> Result<(),String>;  

    // FIXME: add error return objects
    
    fn write_data(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, data: &String, remote_path: &String, mode: Option<i32>) -> Result<(),Arc<TaskResponse>>;

    /* 
    FIXME: should add, return result
    fn get_file(&self, remote_path: String) -> String;
    */

    fn run_command(&self, handle: &TaskHandle, request: &Arc<TaskRequest>, cmd: &String) -> Result<Arc<TaskResponse>,Arc<TaskResponse>>;

}