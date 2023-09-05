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

use crate::cli::parser::CliParser;
use crate::connection::no::NoFactory;
use crate::connection::ssh::SshFactory;
use crate::connection::local::LocalFactory;
use crate::playbooks::traversal::{playbook_traversal,RunState};
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::inventory::inventory::Inventory;
use std::sync::{Arc,RwLock};

struct SyntaxVisitor {}
impl SyntaxVisitor {
    pub fn new() -> Self { Self {} }
}
impl PlaybookVisitor for SyntaxVisitor {
    fn is_syntax_only(&self)    -> bool { return true; }
    fn is_check_mode(&self)     -> bool { return true; }
}

struct LiveVisitor {}
impl LiveVisitor {
    pub fn new() -> Self { Self {} }
}
impl PlaybookVisitor for LiveVisitor {
    fn is_syntax_only(&self)    -> bool { return false; }
    fn is_check_mode(&self)     -> bool { return false; }
}

pub fn playbook_syntax_scan(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    let run_state = Arc::new(RunState {
        inventory: Arc::clone(inventory),
        playbook_paths: Arc::clone(&parser.playbook_paths),
        role_paths: Arc::clone(&parser.role_paths),
        context: Arc::new(RwLock::new(PlaybookContext::new(parser))),
        visitor: Arc::new(RwLock::new(SyntaxVisitor::new())),
        connection_factory: Arc::new(RwLock::new(NoFactory::new())),
    });
    return match playbook_traversal(&run_state) {
        Ok(_)  => run_state.visitor.read().unwrap().get_exit_status(&run_state.context),
        Err(s) => { println!("{}", s); 1 }
    };
}

pub fn playbook_ssh(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    let run_state = Arc::new(RunState {
        inventory: Arc::clone(inventory),
        playbook_paths: Arc::clone(&parser.playbook_paths),
        role_paths: Arc::clone(&parser.role_paths),
        context: Arc::new(RwLock::new(PlaybookContext::new(parser))),
        visitor: Arc::new(RwLock::new(LiveVisitor::new())),
        connection_factory: Arc::new(RwLock::new(SshFactory::new(inventory))),
    });
    return match playbook_traversal(&run_state) {
        Ok(_)  => run_state.visitor.read().unwrap().get_exit_status(&run_state.context),
        Err(s) => { println!("{}", s); 1 }
    };
}

pub fn playbook_local(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    let run_state = Arc::new(RunState {
        inventory: Arc::clone(inventory),
        playbook_paths: Arc::clone(&parser.playbook_paths),
        role_paths: Arc::clone(&parser.role_paths),
        context: Arc::new(RwLock::new(PlaybookContext::new(parser))),
        visitor: Arc::new(RwLock::new(LiveVisitor::new())),
        connection_factory: Arc::new(RwLock::new(LocalFactory::new(inventory))),
    });
    return match playbook_traversal(&run_state) {
        Ok(_)  => run_state.visitor.read().unwrap().get_exit_status(&run_state.context),
        Err(s) => { println!("{}", s); 1 }
    };
}

// FIXME: add check modes for SSH and local