
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
use crate::tasks::*;
use std::sync::RwLock;
use crate::inventory::hosts::Host;
use inline_colorization::{color_red,color_blue,color_green,color_cyan,color_reset,color_yellow};
use crate::connection::command::CommandResult;
use crate::playbooks::traversal::HandlerMode;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::fs::File;
use serde_json::json;
use guid_create::GUID;
use chrono::prelude::*;
use std::env;

// visitor contains various functions that are called from all over the program
// to send feedback to the user and logs

#[derive(PartialEq)]
pub enum CheckMode {
    Yes,
    No
}

pub struct PlaybookVisitor {
    pub check_mode: CheckMode,
    pub logfile: Option<Arc<RwLock<File>>>,
    pub run_id: String,
    pub utc_start: DateTime<Utc>
}

pub struct LogData {
    pub event: String,
    pub play: Option<String>,
    pub playbook_path: Option<String>,
    pub role: Option<String>,
    pub task: Option<String>,
    pub task_ct: Option<usize>,
    pub cmd: Option<String>,
    pub cmd_rc: Option<i32>,
    pub cmd_out: Option<String>,
    pub task_status: Option<String>,
    pub host: Option<String>,
    pub summary: Option<serde_json::map::Map<String,serde_json::Value>>
}

impl PlaybookVisitor {

    pub fn new(check_mode: CheckMode) -> Self {

        let logpath : String = match env::var("JET_LOG") {
            Ok(x) => {
                x
            },
            Err(_) => String::from("/var/log/jetp/jetp.log")
        };

        // TODO: make logfile location configurable by environment variable
        let logfile : Option<Arc<RwLock<File>>> = match OpenOptions::new().write(true).append(true).open(logpath) {
            Ok(x) => Some(Arc::new(RwLock::new(x))),
            Err(_) => None
        };

        let s = Self {
            check_mode: check_mode,
            logfile: logfile,
            utc_start: Utc::now(),
            run_id: GUID::rand().to_string()
        };
        s
    }

    pub fn log_entry(&self, event: &String, context: Arc<RwLock<PlaybookContext>>) -> LogData {
        let ctx = context.read().unwrap();
        LogData {
            event: event.clone(),
            play: ctx.play.clone(),
            playbook_path: ctx.playbook_path.clone(),
            role: match &ctx.role {
                Some(x) => Some(x.name.clone()),
                None => None
            },
            task: match &ctx.task {
                Some(x) => Some(x.clone()),
                None => None
            },
            task_ct: match &ctx.task {
                Some(_) => Some(ctx.task_count),
                None => None
            },
            cmd: None,
            cmd_rc: None,
            cmd_out: None,
            task_status: None,
            host: None,
            summary: None
        }
    }

    pub fn log(&self, log: &LogData) {

        if self.logfile.is_none() {
            return;
        }

        let now = Utc::now();

        let mut obj =  serde_json::map::Map::new();
        obj.insert(String::from("event"), json!(log.event.clone()));
        obj.insert(String::from("run"), json!(self.run_id));
        obj.insert(String::from("now"), json!(now.to_rfc2822()));
        obj.insert(String::from("start"), json!(self.utc_start.to_rfc2822()));
        let elapsed = now - self.utc_start;
        obj.insert(String::from("elapsed"),     json!(elapsed.num_seconds()));

        if log.play.is_some()        { obj.insert(String::from("playbook"),    json!(log.playbook_path.clone().unwrap())); }
        if log.play.is_some()        { obj.insert(String::from("play"),        json!(log.play.clone().unwrap()));          }
        if log.role.is_some()        { obj.insert(String::from("role"),        json!(log.role.clone().unwrap()));          }
        if log.task.is_some()        { obj.insert(String::from("task"),        json!(log.task.clone().unwrap()));          }
        if log.task.is_some()        { obj.insert(String::from("task_ct"),     json!(log.task_ct.clone().unwrap()));       }
        if log.cmd.is_some()         { obj.insert(String::from("cmd"),         json!(log.cmd.clone().unwrap()));           }
        if log.cmd_rc.is_some()      { obj.insert(String::from("cmd_rc"),      json!(log.cmd_rc.clone().unwrap()));        }
        if log.cmd_out.is_some()     { obj.insert(String::from("cmd_out"),     json!(log.cmd_out.clone().unwrap()));       }
        if log.task_status.is_some() { obj.insert(String::from("task_status"), json!(log.task_status.clone().unwrap()));   }
        if log.host.is_some()        { obj.insert(String::from("host"),        json!(log.host.clone().unwrap()));          }
        
        if log.summary.is_some()     { obj.insert(String::from("summary"),     json!(log.summary.clone().unwrap()));       }


   
        match serde_json::to_string(&obj) {
            Ok(json_str) => {
                let mut f = self.logfile.as_ref().unwrap().write().unwrap();
                match writeln!(f, "{}",  json_str) {
                    Ok(_) => {},
                    Err(_e) => { }
                }
            },
            Err(_y) => {}
        }

    }

    pub fn is_check_mode(&self) -> bool { 
        return self.check_mode == CheckMode::Yes; 
    }

    pub fn banner(&self) {
        println!("----------------------------------------------------------");
    }

    // used by the echo module
    pub fn debug_host(&self, host: &Arc<RwLock<Host>>, message: &String) {
        println!("{color_cyan}  ..... {} : {}{color_reset}", host.read().unwrap().name, message);
    }

    pub fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let ctx = context.read().unwrap();
        let path = ctx.playbook_path.as_ref().unwrap();
        self.banner();
        println!("> playbook start: {}", path);

        let log_entry = self.log_entry(&String::from("PLAYBOOK_START"), context.clone());
        self.log(&log_entry);
    }

    pub fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let play = &context.read().unwrap().play;
        self.banner();
        println!("> play: {}", play.as_ref().unwrap());

        let log_entry = self.log_entry(&String::from("PLAY_START"), context.clone());
        self.log(&log_entry);

    }

    pub fn on_role_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let log_entry = self.log_entry(&String::from("ROLE_START"), context.clone());
        self.log(&log_entry);
    }

    pub fn on_role_stop(&self, _context: &Arc<RwLock<PlaybookContext>>) {
    }

    pub fn on_play_stop(&self, context: &Arc<RwLock<PlaybookContext>>, failed: bool) {
        // failed occurs if *ALL* hosts in a play have failed
        let ctx = context.read().unwrap();
        let play_name = ctx.get_play_name();
        if ! failed {
            self.banner();
            println!("> play complete: {}", play_name);
        } else {
            self.banner();
            println!("{color_red}> play failed: {}{color_reset}", play_name);

        }
    }

    pub fn on_exit(&self, context: &Arc<RwLock<PlaybookContext>>) {
        println!("----------------------------------------------------------");
        println!("");
        self.show_playbook_summary(context);
    }

    pub fn on_task_start(&self, context: &Arc<RwLock<PlaybookContext>>, is_handler: HandlerMode) {
        let context2 = context.read().unwrap();
        let task = context2.task.as_ref().unwrap();
        let role = &context2.role;

        let what = match is_handler {
            HandlerMode::NormalTasks => String::from("task"),
            HandlerMode::Handlers    => String::from("handler")
        };

        self.banner();
        if role.is_none() {
            println!("> begin {}: {}", what, task);
        }
        else {
            println!("> ({}) begin {}: {}", role.as_ref().unwrap().name, what, task);
        }

        let log_entry = self.log_entry(&String::from("TASK_START"), Arc::clone(context));
        self.log(&log_entry);
    }

    pub fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        self.banner();
        println!("> batch {}/{}, {} hosts", batch_num+1, batch_count, batch_size);
    }

    pub fn on_host_task_start(&self, _context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("… {} => running", host2.name);
    }

    pub fn on_notify_handler(&self, host: &Arc<RwLock<Host>>, which_handler: &String) {
        let host2 = host.read().unwrap();
        println!("… {} => notified: {}", host2.name, which_handler);
    }

    pub fn on_host_delegate(&self, host: &Arc<RwLock<Host>>, delegated: &String) {
        let host2 = host.read().unwrap();
        println!("{color_blue}✓ {} => delegating to: {}{color_reset}",  &host2.name, delegated.clone());
    }

    pub fn on_host_task_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        {
            let mut context2 = context.write().unwrap();
            context2.increment_attempted_for_host(&host2.name);
            match &task_response.status {
                TaskStatus::IsCreated  =>  {
                    println!("{color_blue}✓ {} => created{color_reset}",  &host2.name);
                    context2.increment_created_for_host(&host2.name);
                },
                TaskStatus::IsRemoved  =>  {
                    println!("{color_blue}✓ {} => removed{color_reset}",  &host2.name);
                    context2.increment_removed_for_host(&host2.name);
                },
                TaskStatus::IsModified =>  {
                    let changes2 : Vec<String> = task_response.changes.iter().map(|x| { format!("{:?}", x) }).collect();
                    let change_str = changes2.join(",");
                    println!("{color_blue}✓ {} => modified ({}){color_reset}", &host2.name, change_str);
                    context2.increment_modified_for_host(&host2.name);
                },
                TaskStatus::IsExecuted =>  {
                    println!("{color_blue}✓ {} => complete{color_reset}", &host2.name);
                    context2.increment_executed_for_host(&host2.name);
                },
                TaskStatus::IsPassive  =>  {
                    // println!("{color_green}! host: {} => ok (no effect) {color_reset}", &host2.name);
                    context2.increment_passive_for_host(&host2.name);
                }
                TaskStatus::IsMatched  =>  {
                    println!("{color_green}✓ {} => matched {color_reset}", &host2.name);
                    context2.increment_matched_for_host(&host2.name);
                }
                TaskStatus::IsSkipped  =>  {
                    println!("{color_yellow}✓ {} => skipped {color_reset}", &host2.name);
                    context2.increment_skipped_for_host(&host2.name);
                }
                TaskStatus::Failed => {
                    println!("{color_yellow}✓ {} => failed (ignored){color_reset}", &host2.name);
                }
                _ => {
                    panic!("on host {}, invalid final task return status, FSM should have rejected: {:?}", host2.name, task_response); 
                }
            }
        }

        let mut log_entry = self.log_entry(&String::from("TASK_STATUS"), Arc::clone(context));
        log_entry.host = Some(host2.name.clone());
        log_entry.task_status = Some(format!("{:?}", &task_response.status));
        self.log(&log_entry);

    }

    // the check mode version of on_host_task_ok - different possible states, slightly different output

    pub fn on_host_task_check_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        {
            let mut context2 = context.write().unwrap();
            context2.increment_attempted_for_host(&host2.name);
            match &task_response.status {
                TaskStatus::NeedsCreation  =>  {
                    println!("{color_blue}✓ {} => would create{color_reset}",  &host2.name);
                    context2.increment_created_for_host(&host2.name);
                },
                TaskStatus::NeedsRemoval  =>  {
                    println!("{color_blue}✓ {} => would remove{color_reset}",  &host2.name);
                    context2.increment_removed_for_host(&host2.name);
                },
                TaskStatus::NeedsModification =>  {
                    let changes2 : Vec<String> = task_response.changes.iter().map(|x| { format!("{:?}", x) }).collect();
                    let change_str = changes2.join(",");
                    println!("{color_blue}✓ {} => would modify ({}) {color_reset}", &host2.name, change_str);
                    context2.increment_modified_for_host(&host2.name);
                },
                TaskStatus::NeedsExecution =>  {
                    println!("{color_blue}✓ {} => would run{color_reset}", &host2.name);
                    context2.increment_executed_for_host(&host2.name);
                },
                TaskStatus::IsPassive  =>  {
                    context2.increment_passive_for_host(&host2.name);
                }
                TaskStatus::IsMatched  =>  {
                    println!("{color_green}✓ {} => matched {color_reset}", &host2.name);
                    context2.increment_matched_for_host(&host2.name);
                }
                TaskStatus::IsSkipped  =>  {
                    println!("{color_yellow}✓ {} => skipped {color_reset}", &host2.name);
                    context2.increment_skipped_for_host(&host2.name);
                }
                TaskStatus::Failed => {
                    println!("{color_yellow}✓ {} => failed (ignored){color_reset}", &host2.name);
                }
                _ => {
                    panic!("on host {}, invalid check-mode final task return status, FSM should have rejected: {:?}", host2.name, task_response); 
                }
            }
        }

        let mut log_entry = self.log_entry(&String::from("TASK_CHECK_STATUS"), Arc::clone(context));
        log_entry.host = Some(host2.name.clone());
        log_entry.task_status = Some(format!("{:?}", &task_response.status));
        self.log(&log_entry);
    }

    pub fn on_host_task_retry(&self, _context: &Arc<RwLock<PlaybookContext>>,host: &Arc<RwLock<Host>>, retries: u64, delay: u64) {
        let host2 = host.read().unwrap();
        println!("{color_blue}! {} => retrying ({} retries left) in {} seconds{color_reset}",host2.name,retries,delay);
    }

    pub fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let mut log_entry = self.log_entry(&String::from("TASK_FAILED"), Arc::clone(context));
        let host2 = host.read().unwrap();
        if task_response.msg.is_some() {
            let msg = &task_response.msg;
            if task_response.command_result.is_some() {
                {
                    let cmd_result = task_response.command_result.as_ref().as_ref().unwrap();
                    let _lock = context.write().unwrap();
                    println!("{color_red}! {} => failed", host2.name);
                    println!("    cmd: {}", cmd_result.cmd);
                    println!("    out: {}", cmd_result.out);
                    println!("    rc: {}{color_reset}", cmd_result.rc);
                    log_entry.cmd     = Some(cmd_result.cmd.clone());
                    log_entry.cmd_out = Some(cmd_result.out.clone());
                    log_entry.cmd_rc  = Some(cmd_result.rc.clone());
                }
            } else {
                println!("{color_red}! error: {}: {}{color_reset}", host2.name, msg.as_ref().unwrap());
            }
        } else {
            println!("{color_red}! host failed: {}, {color_reset}", host2.name);
        }

        context.write().unwrap().increment_failed_for_host(&host2.name);
        log_entry.host = Some(host2.name.clone());
        log_entry.task_status = Some(format!("{:?}", &task_response.status));
        self.log(&log_entry);
    }

    pub fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        context.write().unwrap().increment_failed_for_host(&host2.name);
        println!("{color_red}! connection failed to host: {}{color_reset}", host2.name);
        let mut log_entry = self.log_entry(&String::from("HOST_CONNECT_FAILED"), Arc::clone(context));
        log_entry.host = Some(host2.name.clone());
        self.log(&log_entry);
    }

    pub fn get_exit_status(&self, context: &Arc<RwLock<PlaybookContext>>) -> i32 {
        let failed_hosts = context.read().unwrap().get_hosts_failed_count();
        return match failed_hosts {
            0 => 0,
            _ => 1
        };
    }
    
    pub fn on_before_transfer(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, path: &String) {
        let host2 = host.read().unwrap();
        if context.read().unwrap().verbosity > 0 {
            println!("{color_blue}! {} => transferring to: {}", host2.name, &path.clone());
        }
    }

    pub fn on_command_run(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, cmd: &String) {
        let host2 = host.read().unwrap();
        if context.read().unwrap().verbosity > 0 {
            println!("{color_blue}! {} => exec: {}", host2.name, &cmd.clone());
        }
    }

    pub fn on_command_ok(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().unwrap();
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 2 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_blue}! {} ... command ok", host2.name);
            println!("    cmd: {}", cmd_result.cmd);           
            println!("    out: {}", cmd_result.out.clone());
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }

    pub fn on_command_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>, result: &Arc<Option<CommandResult>>,) {
        let host2 = host.read().expect("context read");
        let cmd_result = result.as_ref().as_ref().expect("missing command result");
        if context.read().unwrap().verbosity > 2 {
            let _ctx2 = context.write().unwrap(); // lock for multi-line output
            println!("{color_red}! {} ... command failed", host2.name);
            println!("    cmd: {}", cmd_result.cmd);
            println!("    out: {}", cmd_result.out.clone());
            println!("    rc: {}{color_reset}", cmd_result.rc);
        }
    }

    pub fn show_playbook_summary(&self, context: &Arc<RwLock<PlaybookContext>>) {

        let ctx = context.read().unwrap();

        let seen_hosts = ctx.get_hosts_seen_count();
        let role_ct = ctx.get_role_count();
        let task_ct = ctx.get_task_count();
        let action_ct = ctx.get_total_attempted_count();
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
        let matched_ct = ctx.get_total_matched_count();
        let matched_hosts = ctx.get_hosts_matched_count();
        let skipped_ct = ctx.get_total_skipped_count();
        let skipped_hosts = ctx.get_hosts_skipped_count();
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
                          | Matched | {matched_ct} | {matched_hosts}\n\
                          | Created | {created_ct} | {created_hosts}\n\
                          | Modified | {modified_ct} | {modified_hosts}\n\
                          | Removed | {removed_ct} | {removed_hosts}\n\
                          | Executed | {executed_ct} | {executed_hosts}\n\
                          | Passive | {passive_ct} | {passive_hosts}\n\
                          | Skipped | {skipped_ct} | {skipped_hosts}\n\
                          | --- | --- | ---\n\
                          | Unchanged | {unchanged_ct} | {unchanged_hosts}\n\
                          | Changed | {adjusted_ct} | {adjusted_hosts}\n\
                          | Failed | {failed_ct} | {failed_hosts}\n\
                          |-|-|-");

        crate::util::terminal::markdown_print(&mode_table);
        println!("{}", format!("\n{summary}"));
        println!("");

        let mut log_entry = self.log_entry(&String::from("SUMMARY"), Arc::clone(context));
        let mut map : serde_json::map::Map<String,serde_json::Value> = serde_json::map::Map::new();
        map.insert(String::from("matched_ct"),      json!(matched_ct));
        map.insert(String::from("matched_hosts"),   json!(matched_hosts));
        map.insert(String::from("created_ct"),      json!(created_ct));
        map.insert(String::from("created_hosts"),   json!(created_hosts));
        map.insert(String::from("modified_ct"),     json!(modified_ct));
        map.insert(String::from("modified_hosts"),  json!(modified_hosts));
        map.insert(String::from("executed_ct"),     json!(executed_ct));
        map.insert(String::from("executed_hosts"),  json!(executed_hosts));
        map.insert(String::from("passive_ct"),      json!(passive_ct));
        map.insert(String::from("passive_hosts"),   json!(passive_hosts));
        map.insert(String::from("skipped_ct"),      json!(skipped_ct));
        map.insert(String::from("skipped_hosts"),   json!(skipped_hosts));
        map.insert(String::from("unchanged_ct"),    json!(unchanged_ct));
        map.insert(String::from("unchanged_hosts"), json!(unchanged_hosts));
        map.insert(String::from("adjusted_ct"),     json!(adjusted_ct));
        map.insert(String::from("adjusted_hosts"),  json!(adjusted_hosts));
        map.insert(String::from("failed_ct"),       json!(failed_ct));
        map.insert(String::from("failed_hosts"),    json!(failed_hosts));
        log_entry.summary = Some(map.clone());
        self.log(&log_entry);

    }

}