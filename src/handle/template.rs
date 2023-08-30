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

use std::sync::{Arc,Mutex,RwLock};
use std::path::{Path,PathBuf};
use crate::connection::connection::Connection;
use crate::connection::command::{CommandResult,cmd_info};
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskStatus, TaskResponse};
use crate::inventory::hosts::{Host,HostOSType};
use crate::playbooks::traversal::RunState;
use crate::tasks::fields::Field;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use crate::tasks::FileAttributesEvaluated;
use crate::tasks::cmd_library::{screen_path,screen_general_input_strict,screen_general_input_loose};

#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum Safety {
    Safe,
    Unsafe
}

pub struct Template {
    run_state: Arc<RunState>, 
    connection: Arc<Mutex<dyn Connection>>,
    host: Arc<RwLock<Host>>, 
    handle: Arc<Option<TaskHandle>>,
}

impl TemplateC {

    pub fn new(run_state_handle: Arc<RunState>, connection_handle: Arc<Mutex<dyn Connection>>, host_handle: Arc<RwLock<Host>>) -> Self {
        Self {
            run_state: run_state_handle,
            connection: connection_handle,
            host: host_handle,
            task_handle: Arc::new(None),
        }
    }

    pub fn attach_handle(task_handle: Arc<TaskHandle>) {
        self.handle = Some(task_handle);
    }

    pub fn template_string_unsafe(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // note to module authors:
        // if you have a path, call template_path instead!  Do not call template_str as you will ignore path sanity checks.
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        let result2 = self.unwrap_string_result(request, &result)?;
        return Ok(result2);
    }

    pub fn template_string(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        let result = self.template_string_unsafe(request, field, template);
        return match result {
            Ok(x) => match screen_general_input_strict(&x) {
                Ok(y) => Ok(y),
                Err(z) => { return Err(self.is_failed(request, &format!("field {}, {}", field, z))) }
            },
            Err(y) => Err(y)
        };
    }

    pub fn template_path(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host);
        let result2 = self.unwrap_string_result(request, &result)?;
        return match screen_path(&result2) {
            Ok(x) => Ok(x), Err(y) => { return Err(self.is_failed(request, &format!("{}, for field {}", y, field))) }
        }
    }

    pub fn template_string_option_unsafe(&self, 
        request: &Arc<TaskRequest>, 
        field: &String, 
        template: &Option<String>) 
            -> Result<Option<String>,Arc<TaskResponse>> {

        if template.is_none() { return Ok(None); }
        let result = self.template_string(request, field, &template.as_ref().unwrap());
        return match result { 
            Ok(x) => Ok(Some(x)), 
            Err(y) => { Err(self.is_failed(request, &format!("field ({}) template error: {:?}", field, y))) } 
        };
    }

    pub fn template_string_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        let result = self.template_string_option_unsafe(request, field, template);
        return match result {
            Ok(x1) => match x1 {
                Some(x) => match screen_general_input_strict(&x) {
                    Ok(y) => Ok(Some(y)),
                    Err(z) => { return Err(self.is_failed(request, &format!("field {}, {}", field, z))) }
                },
                None => Ok(None)
            },
            Err(y) => Err(y)
        };
    }

    pub fn template_integer(&self, request: &Arc<TaskRequest>, field: &String, template: &String)-> Result<i64,Arc<TaskResponse>> {
        let st = self.template_string(request, field, template)?;
        let num = st.parse::<i64>();
        return match num {
            Ok(num) => Ok(num), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_integer_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>) -> Result<Option<i64>,Arc<TaskResponse>> {
        if template.is_none() { return Ok(None); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let num = st.parse::<i64>();
        // FIXME: these can use map_err
        return match num {
            Ok(num) => Ok(Some(num)), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn template_boolean(&self, request: &Arc<TaskRequest>, field: &String, template: &String) -> Result<bool,Arc<TaskResponse>> {
        let st = self.template_string(request,field, template)?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn template_boolean_option(&self, request: &Arc<TaskRequest>, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        if template.is_none() { return Ok(false); }
        let st = self.template_string(request, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x), Err(_err) => Err(self.is_failed(request, &format!("field ({}) value is not an boolean: {}", field, st)))
        }
    }

    pub fn test_cond(&self, request: &Arc<TaskRequest>, expr: &String) -> Result<bool, Arc<TaskResponse>> {
        let result = self.get_context().read().unwrap().test_cond(expr, &self.host);
        return match result {
            Ok(x) => Ok(x), Err(y) => Err(self.is_failed(request, &y))
        }
    }

    #[inline]
    pub fn find_template_path(&self, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        return self.find_sub_path(&String::from("templates"), request, field, str_path);
    }

    pub fn find_file_path(&self, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        return self.find_sub_path(&String::from("files"), request, field, str_path);
    }

    pub fn get_desired_numeric_mode(&self, request: &Arc<TaskRequest>, attribs: &Option<FileAttributesEvaluated>) -> Result<Option<i32>,Arc<TaskResponse>>{
        return FileAttributesEvaluated::get_numeric_mode(self, request, attribs); 
    }

    fn find_sub_path(&self, prefix: &String, request: &Arc<TaskRequest>, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        let prelim = match screen_path(&str_path) {
            Ok(x) => x, 
            Err(y) => { return Err(self.is_failed(request, &format!("{}, for field: {}", y, field))) }
        };
        let mut path = PathBuf::new();
        path.push(prelim);
        if path.is_absolute() {
            if path.is_file() {
                return Ok(path);
            } else {
                return Err(self.is_failed(request, &format!("field ({}): no such file: {}", field, str_path)));
            }
        } else {
            let mut path2 = PathBuf::new();
            path2.push(prefix);
            path2.push(str_path);
            if path2.is_file() {
                return Ok(path2);
            } else {
                return Err(self.is_failed(request, &format!("field field ({}): no such file: {}", field, str_path)));
            }
        }
    }


}