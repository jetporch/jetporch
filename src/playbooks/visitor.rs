
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
// ABOUT: visitor.rs
// these functions may be thought of as callbacks that report what is going on
// with playbook code.  Eventually these themselves may take a vector of additional
// functions, but the plan for now is that they would be overriden in the cli/*.rs
// commands when custom behavior was needed.
// ===================================================================================


use crate::playbooks::context::PlaybookContext;
use crate::tasks::response::{TaskResponse,TaskStatus};
use std::sync::Arc;
use std::sync::RwLock;
use crate::util::terminal::two_column_table;
use crate::inventory::hosts::Host;

pub trait PlaybookVisitor {

    fn banner(&self) {
        println!("----------------------------------------------------------");
    }

    fn debug(&self, message: String) {
        println!("| debug | {}", message.clone());
    }

    fn debug_host(&self, host: &Arc<RwLock<Host>>, message: String) {
        println!("     > {} : {}", host.read().unwrap().name, message.clone());
    }

    fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.playbook_path.lock().unwrap();
        //let path = arc.as_ref().unwrap();
        let path = "<PATH GOES HERE>".to_string();
        self.banner();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        let play = &context.read().unwrap().play;
        self.banner();
        println!("> play start: {}", play.as_ref().unwrap());
    }
    
    fn on_role_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        //let role = arc.as_ref().unwrap();
        let role = &context.read().unwrap().role;
        self.banner();
        println!("> role start: {}", role.as_ref().unwrap());
    }

    fn on_role_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        let role = &context.read().unwrap().role;
        self.banner();
        println!("> role stop: {}", role.as_ref().unwrap());
    }

    fn on_play_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        
        let ctx = context.read().unwrap();
        let play_name = ctx.get_play_name();

        if self.is_syntax_only() {

            let elements: Vec<(String,String)> = vec![     
                (String::from("Roles"), format!("{}", ctx.get_role_count())),
                (String::from("Tasks"), format!("{}", ctx.get_task_count())),
                (String::from("OK"), String::from("Syntax ok. No configuration attempted.")),
            ];
            two_column_table(String::from("Play Result"), play_name.clone(), elements);

        } else {
            println!("(full play output not implemented)");
        }
    }

    fn on_task_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.task.lock().unwrap();
        //let task = arc.as_ref().unwrap();
        //let module = task.get_module();
        let context = context.read().unwrap();
        //let play = context.play;
        let task = context.task.as_ref().unwrap();
        self.banner();
        println!("> begin task: {}", task);
    }

    fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        self.banner();
        println!("> batch {}/{}, {} hosts", batch_num+1, batch_count, batch_size);
    }

    fn on_task_stop(&self, _context: &Arc<RwLock<PlaybookContext>>) {
        /*
        let context = context.read().unwrap();
        let host = context.host
        let play = context.play;
        let task = context.task;
        println!("@ task complete: {}", task.as_ref().unwrap());
        */
    }

    fn on_host_task_start(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("! host: {} ...", host2.name);
    }

    fn on_host_task_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        match &task_response.status {
            TaskStatus::IsCreated => { println!("! host: {} => created", host2.name); },
            TaskStatus::IsRemoved => { println!("! host: {} => removed", host2.name); },
            TaskStatus::IsModified => { println!("! host: {} => modified", host2.name); },
            TaskStatus::IsChanged => { println!("! host: {} => changed", host2.name); },
            TaskStatus::IsExecuted => { println!("! host: {} => executed", host2.name); },
            _ => { panic!("on host {}, invalid final task return status, FSM should have rejected: {:?}", host2.name, task_response); }
        }
    }

    fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("! host failed: {}", host2.name);
        //println!("> task failed on host: {}", host);
    }

    fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("! connection failed to host: {}", host2.name);
    }

    fn is_syntax_only(&self) -> bool;

    fn is_check_mode(&self) -> bool;

}