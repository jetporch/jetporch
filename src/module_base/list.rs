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


use serde::Deserialize;

use crate::playbooks::language::{AsInteger};

pub trait TaskProperties {
    fn get_name(&self) -> String;
    fn get_when(&self) -> String;
    fn get_changed_when(&self) -> String;
    fn get_retry(&self) -> String;
    fn get_delay(&self) -> String;
    fn get_register(&self) -> String;
}

/** ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (1) **/
use crate::module_library::echo::Echo;

pub trait IsTask: TaskProperties { 
    /*
    fn get_module(&self) -> String;
    fn run(&self) -> Result<(), String>;
    */
}


#[derive(Deserialize,Debug)]
//#[serde(tag="module", rename_all="lowercase")]
#[serde(rename_all="lowercase")]
pub enum Task {
    /** ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (2) **/
    Echo(Echo),
    /* Shell(Shell), */
}

// maybe macros later, but right now they are hurting things
impl Task {

    pub fn get_name(&self) -> String { 
        return match self {
            /** ADD NEW MODULES HERE, DE-ALPHABETIZE ON PENALTY OF DEATH (3) **/
            Task::Echo(x) => x.get_name(), 
            _ => { panic!("internal error"); },
        };
    }

}

    //return self.name.clone() }
    
    /*
    fn get_when(&self) -> Option<String> { return self.when.clone() } 
        fn get_changed_when(&self) -> Option<String> { return self.changed_when.clone() }
        fn get_retry(&self) -> Option<AsInteger> { return Some(AsInteger::Integer(0)); }
        fn get_delay(&self) -> Option<AsInteger> { return Some(AsInteger::Integer(0)); }
        fn get_register(&self) -> Option<String> { return self.register.clone(); }
    } 
    */ 



