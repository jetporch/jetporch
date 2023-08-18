
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
use crate::tasks::response::TaskResponse;
use std::sync::Arc;
use std::sync::RwLock;
use crate::util::terminal::two_column_table;
use crate::inventory::hosts::Host;

pub trait PlaybookVisitor {

    fn debug(&self, message: String) {
        println!("> debug: {}", message.clone());
    }

    fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.playbook_path.lock().unwrap();
        //let path = arc.as_ref().unwrap();
        let path = "FIXME".to_string();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        let play = "FIXME".to_string();

        println!("> play start: {}", play);
    }
    
    fn on_role_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        //let role = arc.as_ref().unwrap();
        let role = "FIXME".to_string();
        println!("> role start: {}", role);
    }

    fn on_role_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        let role = "FIXME".to_string();
        println!("> role stop: {}", role);
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
        let task = "FIXME".to_string();
        println!("> task start: {}", task);
    }

    fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        println!("> batch {}/{}, {} hosts", batch_num, batch_count, batch_size);
    }

    fn on_task_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.task.lock().unwrap();
        //let task = arc.as_ref().unwrap();
        let task = "FIXME".to_string();
        println!("> task complete: {}", task);
    }

    fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("> host task failed: {}", host2.name);
        //println!("> task failed on host: {}", host);
    }

    fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("> connection failed to host: {}", host2.name);
    }

    fn is_syntax_only(&self) -> bool;

    fn is_check_mode(&self) -> bool;

}