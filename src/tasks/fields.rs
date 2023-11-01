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

use std::vec::Vec;

// this is to prevent typos in module code between Query & Modify 
// match legs vs using strings

// KEEP THESE ALPHABETIZED

#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum Field {
    Branch,
    Content,
    Disable,
    Enable,
    Gecos,
    Gid,
    Group,
    Groups,
    Mode,
    Owner,
    Restart,
    Shell,
    Start,
    Stop,
    Uid,
    Version,
}

impl Field {
    pub fn all_file_attributes() -> Vec<Field> {
        let mut result : Vec<Field> = Vec::new();
        result.push(Field::Owner);
        result.push(Field::Group);
        result.push(Field::Mode);
        return result; 
    }
}
