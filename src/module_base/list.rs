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

use serde::{Deserialize};

// all internal modules here alphabetized here (and below)
// unused imports warnings are disabled here because of macros - they are used

#[allow(unused_imports)]
use crate::module_library::external::{External};
#[allow(unused_imports)]
use crate::module_library::include::{Include};
#[allow(unused_imports)]
use crate::module_library::shell::{Shell};

// all internal modules here alphabetized


#[derive(Debug,Deserialize,PartialEq)]
#[serde(tag="module", rename_all="lowercase")]
//#[serde(untagged,deny_unknown_fields)]
pub enum Task {
    External,
    Include,
    Shell,
}