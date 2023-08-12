
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

pub trait PlaybookVisitor {

    fn debug(&self, message: String) {
        println!("> debug: {}", message.clone());
    }

    fn on_playbook_start(&self, context: &PlaybookContext) {
        let arc = context.playbook_path.lock().unwrap();
        let path = arc.as_ref().unwrap();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &PlaybookContext) {
        let arc = context.play.lock().unwrap();
        let play = arc.as_ref().unwrap();
        println!("> play start: {}", play);
    }
    
    fn on_role_start(&self, context: &PlaybookContext) {
        let arc = context.role_name.lock().unwrap();
        let role = arc.as_ref().unwrap();
        println!("> role start: {}", role);
    }

    fn on_role_stop(&self, context: &PlaybookContext) {
        let arc = context.role_name.lock().unwrap();
        let role = arc.as_ref().unwrap();
        println!("> role stop: {}", role);
    }

    fn on_play_stop(&self, context: &PlaybookContext) {
        let arc = context.play.lock().unwrap();
        let play = arc.as_ref().unwrap();
        println!("> play complete: {}", play);
    }

    fn on_task_start(&self, context: &PlaybookContext) {
        let arc = context.task.lock().unwrap();
        let task = arc.as_ref().unwrap();
        //let module = task.get_module();
        println!("> task start: {}", task);
    }

    fn on_task_stop(&self, context: &PlaybookContext) {
        let arc = context.task.lock().unwrap();
        let task = arc.as_ref().unwrap();
        println!("> task complete: {}", task);
    }

    

    // TODO: functions for loading in variables and such.

}