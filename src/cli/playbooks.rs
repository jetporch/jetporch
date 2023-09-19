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

use crate::connection::ssh::SshFactory;
use crate::connection::local::LocalFactory;
use crate::connection::no::NoFactory;
use crate::playbooks::traversal::{playbook_traversal,RunState};
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::inventory::inventory::Inventory;
use std::sync::{Arc,RwLock};

// code behind *most* playbook related CLI commands, launched from main.rs

// FIXME: the original plan was for visitors to be able to override more behavior
// than just check mode, but so far we are *not* using it. We can
// probably move to merging visitor with
// context, and using the CheckMode enum when constructing context, 
// eliminating some weird interactions
// when sometimes visitor calls context and in other times outside code
// calls context.

struct CheckVisitor {}
impl CheckVisitor {
    pub fn new() -> Self { Self {} }
}
impl PlaybookVisitor for CheckVisitor {
    fn is_check_mode(&self)     -> bool { return true; }
}

struct LiveVisitor {}
impl LiveVisitor {
    pub fn new() -> Self { Self {} }
}
impl PlaybookVisitor for LiveVisitor {
    fn is_check_mode(&self)     -> bool { return false; }
}

enum CheckMode {
    Yes,
    No
}

enum ConnectionMode {
    Ssh,
    Local,
    Simulate
}

pub fn playbook_ssh(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    return playbook(inventory, parser, CheckMode::No, ConnectionMode::Ssh);
}

pub fn playbook_check_ssh(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    return playbook(inventory, parser, CheckMode::Yes, ConnectionMode::Ssh);
}

pub fn playbook_local(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    return playbook(inventory, parser, CheckMode::No, ConnectionMode::Local);
}

pub fn playbook_check_local(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    return playbook(inventory, parser, CheckMode::Yes, ConnectionMode::Local);
}

pub fn playbook_simulate(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser) -> i32 {
    return playbook(inventory, parser, CheckMode::No, ConnectionMode::Simulate);
}

fn playbook(inventory: &Arc<RwLock<Inventory>>, parser: &CliParser, check_mode: CheckMode, connection_mode: ConnectionMode) -> i32 {
    let run_state = Arc::new(RunState {
        // every object gets an inventory, though with local modes it's empty.
        inventory: Arc::clone(inventory),
        playbook_paths: Arc::clone(&parser.playbook_paths),
        role_paths: Arc::clone(&parser.role_paths),
        limit_hosts: parser.limit_hosts.clone(),
        limit_groups: parser.limit_groups.clone(),
        batch_size: parser.batch_size.clone(),
        // the context is constructed with an instance of the parser instead of having a back-reference
        // to run-state.  Context should mostly *not* get parameters from the parser unless they
        // are going to appear in variables.
        context: Arc::new(RwLock::new(PlaybookContext::new(parser))),
        visitor: match check_mode {
            CheckMode::Yes => Arc::new(RwLock::new(CheckVisitor::new())),
            CheckMode::No => Arc::new(RwLock::new(LiveVisitor::new())),
        },
        connection_factory: match connection_mode {
            ConnectionMode::Ssh => Arc::new(RwLock::new(SshFactory::new(inventory))),
            ConnectionMode::Local => Arc::new(RwLock::new(LocalFactory::new(inventory))),
            ConnectionMode::Simulate => Arc::new(RwLock::new(NoFactory::new()))
        },
        tags: parser.tags.clone(),
        allow_localhost_delegation: parser.allow_localhost_delegation
    });
    return match playbook_traversal(&run_state) {
        Ok(_)  => run_state.visitor.read().unwrap().get_exit_status(&run_state.context),
        Err(s) => { println!("{}", s); 1 }
    };
}

