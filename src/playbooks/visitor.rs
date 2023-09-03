
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

use crate::playbooks::context::PlaybookContext;
use std::sync::Arc;
use crate::util::terminal::two_column_table;
use crate::tasks::*;
use std::sync::RwLock;
use crate::inventory::hosts::Host;
use inline_colorization::{color_red,color_blue,color_green,color_cyan,color_reset,color_yellow};
use std::marker::{Send,Sync};
use crate::connection::command::CommandResult;

// the visitor is a trait with lots of default implementation that can be overridden
// for various CLI commands. It is called extensively during playbook traversal

pub trait PlaybookVisitor : Send + Sync {

    fn banner(&self) {
        println!("----------------------------------------------------------");
    }

    fn debug(&self, message: &String) {
        println!("{color_cyan}  ..... (debug) : {}{color_reset}", message);
    }

    fn debug_host(&self, host: &Arc<RwLock<Host>>, message: &String) {
        println!("{color_cyan}  ..... {} : {}{color_reset}", host.read().unwrap().name, message);
    }

    fn debug_lines(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, messages: &Vec<String>) {
        let _lock = context.write().unwrap();
        for message in messages.iter() {
            self.debug_host(host, &message);
        }
    }

    fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.playbook_path.lock().unwrap();
        let ctx = context.read().unwrap();
        let path = ctx.playbook_path.as_ref().unwrap();
        self.banner();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        let play = &context.read().unwrap().play;
        self.banner();
        println!("> play: {}", play.as_ref().unwrap());
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

    fn on_play_stop(&self, context: &Arc<RwLock<PlaybookContext>>, failed: bool) {
        // failed occurs if *ALL* hosts in a play have failed
        let ctx = context.read().unwrap();
        let play_name = ctx.get_play_name();
        if ! failed {
            if ! self.is_syntax_only() {
                 self.banner();
                 println!("> play complete: {}", play_name);
            }
        } else {
            self.banner();
            println!("{color_red}> play failed: {}{color_reset}", play_name);

        }
    }

    fn on_exit(&self, context: &Arc<RwLock<PlaybookContext>>) -> () {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();

        if self.is_syntax_only() {
            let ctx = context.read().unwrap();
            //let play_name = ctx.get_play_name();


            let (summary1, summary2) : (String,String) = match ctx.get_hosts_failed_count() {
                0 => (format!("Ok"), 
                      format!("No configuration or variable evaluation was attempted.")),
                _ => (format!("Failed"), 
                      format!("{} errors", ctx.failed_tasks))
            };

            let elements: Vec<(String,String)> = vec![
                (String::from("Roles"), format!("{}", ctx.get_role_count())),
                (String::from("Tasks"), format!("{}", ctx.get_task_count())),
                (summary1.clone(), summary2.clone()),
            ];
            println!("");
            two_column_table(&String::from("Syntax Check"), &String::from("..."), &elements);
            // playbook would have failed earlier were it not ok.
        } else {
            println!("----------------------------------------------------------");
            println!("");
            show_playbook_summary(context);
        }
    }

    fn on_task_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        if self.is_syntax_only() { 
            return;
        }
        let context = context.read().unwrap();
        let task = context.task.as_ref().unwrap();
        self.banner();
        println!("> begin task: {}", task);
    }

    fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        if self.is_syntax_only() { 
            return;
        }
        self.banner();
        println!("> batch {}/{}, {} hosts", batch_num+1, batch_count, batch_size);
    }

    fn on_task_stop(&self, _context: &Arc<RwLock<PlaybookContext>>) {
    }

    fn on_host_task_start(&self, _context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        if self.is_syntax_only() { 
            return;
        }
        let host2 = host.read().unwrap();
        println!("… {} => running", host2.name);
    }

    // FIXME: this pattern of the visitor accessing the context is cleaner than the FSM code that accesses both in sequence, so do
    // more of this below.

    fn on_host_task_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        let mut context = context.write().unwrap();
        context.increment_attempted_for_host(&host2.name);
        match &task_response.status {
            TaskStatus::IsCreated  =>  {
                println!("{color_blue}✓ {} => created{color_reset}",  &host2.name);
                context.increment_created_for_host(&host2.name);
            },
            TaskStatus::IsRemoved  =>  {
                println!("{color_blue}✓ {} => removed{color_reset}",  &host2.name);
                context.increment_removed_for_host(&host2.name);
            },
            TaskStatus::IsModified =>  {
                println!("{color_blue}✓ {} => modified{color_reset}", &host2.name);
                context.increment_modified_for_host(&host2.name);
            },
            TaskStatus::IsExecuted =>  {
                println!("{color_blue}✓ {} => complete{color_reset}", &host2.name);
                context.increment_executed_for_host(&host2.name);
            },
            TaskStatus::IsPassive  =>  {
                // println!("{color_green}! host: {} => ok (no effect) {color_reset}", &host2.name);
                context.increment_passive_for_host(&host2.name);
            }
            TaskStatus::IsMatched  =>  {
                println!("{color_green}✓ {} => perfect {color_reset}", &host2.name);
            }
            TaskStatus::IsSkipped  =>  {
                println!("{color_yellow}✓ {} => skipped {color_reset}", &host2.name);
            }
            _ => { panic!("on host {}, invalid final task return status, FSM should have rejected: {:?}", host2.name, task_response); }
        }
    }

    fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        if self.is_syntax_only() { 
            let context = context.read().unwrap();
            let task = context.task.as_ref().unwrap();
            println!("> task: {}", task);
        }
        let host2 = host.read().unwrap();
        if task_response.msg.is_some() {
            let msg = &task_response.msg;
            if task_response.command_result.is_some() {
                {
                    let cmd_result = task_response.command_result.as_ref().as_ref().unwrap();
                    let _lock = context.write().unwrap();
                    // FIXME: add similar output for verbose modes above
                    println!("{color_red}! {} => failed", host2.name);
                    println!("    cmd: {}", cmd_result.cmd);
                    println!("    out: {}", cmd_result.out);
                    println!("    rc: {}{color_reset}", cmd_result.rc);
                }
            } else {
                if self.is_syntax_only() {
                    println!("{color_red}! error: {}{color_reset}", msg.as_ref().unwrap());
                } else {
                    println!("{color_red}! error: {}: {}{color_reset}", host2.name, msg.as_ref().unwrap());
                }
            }
        } else {
            println!("{color_red}! host failed: {}, {color_reset}", host2.name);
        }

        context.write().unwrap().increment_failed_for_host(&host2.name);
        //println!("> task failed on host: {}", host);
    }

    fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        context.write().unwrap().increment_failed_for_host(&host2.name);
        println!("{color_red}! connection failed to host: {}{color_reset}", host2.name);
    }

    fn get_exit_status(&self, context: &Arc<RwLock<PlaybookContext>>) -> i32 {
        let failed_hosts = context.read().unwrap().get_hosts_failed_count();
        return match failed_hosts {
            0 => 0,
            _ => 1
        };
    }

    fn on_command_ok(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().expect("host read");
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 1 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_blue}! {} ... command ok", host2.name);
            println!("    cmd: {}", cmd_result.cmd);
            println!("    out: {}", cmd_result.out);
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }

    fn on_command_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().expect("context read");
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 1 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_red}! {} ... command failed", host2.name);
            println!("    cmd: {}", cmd_result.cmd);
            println!("    out: {}", cmd_result.out);
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }


    fn is_syntax_only(&self) -> bool;

    fn is_check_mode(&self) -> bool;

}



pub fn show_playbook_summary(context: &Arc<RwLock<PlaybookContext>>) {

    let ctx = context.read().unwrap();

    let seen_hosts = ctx.get_hosts_seen_count();
    let role_ct = ctx.get_role_count();
    let task_ct = ctx.get_task_count();
    let action_ct = ctx.get_total_attempted_count();
    //let action_hosts = ctx.get_hosts_attempted_count();
    let created_ct = ctx.get_total_creation_count();
    let created_hosts = ctx.get_hosts_creation_count();
    let modified_ct = ctx.get_total_modified_count();
    let modified_hosts = ctx.get_hosts_modified_count();
    let removed_ct = ctx.get_total_removal_count();
    let removed_hosts = ctx.get_hosts_removal_count();
    let executed_ct = ctx.get_total_executions_count();
    let executed_hosts = ctx.get_hosts_executions_count();
    let passive_ct = ctx.get_total_passive_count();
    let passive_hosts = ctx.get_hosts_passive_count();
    let adjusted_ct = ctx.get_total_adjusted_count();
    let adjusted_hosts = ctx.get_hosts_adjusted_count();
    let unchanged_hosts = seen_hosts - adjusted_hosts;
    let unchanged_ct = action_ct - adjusted_ct;
    let failed_ct    = ctx.get_total_failed_count();
    let failed_hosts = ctx.get_hosts_failed_count();



    let summary = match failed_hosts {
        0 => match adjusted_hosts {
            0 => String::from(format!("{color_green}(✓) Perfect. All hosts matched policy.{color_reset}")),
            _ => String::from(format!("{color_blue}(✓) Actions were applied.{color_reset}")),
        },
        _ => String::from(format!("{color_red}(X) Failures have occured.{color_reset}")),
    };

    let mode_table = format!("|:-|:-|:-|\n\
                      | Results | Items | Hosts \n\
                      | --- | --- | --- |\n\
                      | Roles | {role_ct} | |\n\
                      | Tasks | {task_ct} | {seen_hosts}|\n\
                      | --- | --- | --- |\n\
                      | Created | {created_ct} | {created_hosts}\n\
                      | Modified | {modified_ct} | {modified_hosts}\n\
                      | Removed | {removed_ct} | {removed_hosts}\n\
                      | Executed | {executed_ct} | {executed_hosts}\n\
                      | Passive | {passive_ct} | {passive_hosts}\n\
                      | --- | --- | ---\n\
                      | Unchanged | {unchanged_ct} | {unchanged_hosts}\n\
                      | Changed | {adjusted_ct} | {adjusted_hosts}\n\
                      | Failed | {failed_ct} | {failed_hosts}\n\
                      |-|-|-");

    crate::util::terminal::markdown_print(&mode_table);
    println!("{}", format!("\n{summary}"));
    println!("");



}
