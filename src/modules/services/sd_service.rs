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

use crate::tasks::*;
use crate::handle::handle::{TaskHandle,CheckRc};
use crate::tasks::fields::Field;
//#[allow(unused_imports)]
use serde::{Deserialize};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &'static str = "sd_service";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct SystemdServiceTask {
    pub name: Option<String>,
    pub service: String,
    pub enabled: Option<String>,
    pub started: Option<String>,
    pub restart: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}

struct SystemdServiceAction {
    pub service: String,
    pub enabled: Option<bool>,
    pub started: Option<bool>,
    pub restart: bool,
}

#[derive(Clone,PartialEq,Debug)]
struct ServiceDetails {
    enabled: bool,
    started: bool,
}

impl IsTask for SystemdServiceTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(SystemdServiceAction {
                    service:    handle.template.string_no_spaces(request, tm, &String::from("service"), &self.service)?,
                    enabled:    handle.template.boolean_option_default_none(&request, tm, &String::from("enabled"), &self.enabled)?,
                    started:    handle.template.boolean_option_default_none(&request, tm, &String::from("started"), &self.started)?,
                    restart:    handle.template.boolean_option_default_false(&request, tm, &String::from("restart"), &self.restart)?
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?)
            }
        );
    }

}


impl IsAction for SystemdServiceAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {
    
        match request.request_type {

            TaskRequestType::Query => {

                let mut changes : Vec<Field> = Vec::new();
                let actual = self.get_service_details(handle, request)?; 

                match (actual.enabled, self.enabled) {
                    (true, Some(false)) => { changes.push(Field::Disable); },
                    (false, Some(true)) => { changes.push(Field::Enable);  },
                    _  => {}
                };

                match (actual.started, self.started, self.restart) {
                    (_,     Some(false), true)   => { return Err(handle.response.is_failed(request, &String::from("started:false and restart:true conflict"))); },
                    (true,  Some(true),  true)   => { changes.push(Field::Restart); },
                    (true,  None,        true)   => { changes.push(Field::Restart); /* a little weird, but we know what you mean */ },
                    (false, None,        true)   => { changes.push(Field::Start);   /* a little weird, but we know what you mean */ },
                    (false, Some(true),  _)      => { changes.push(Field::Start); },
                    (true,  Some(false), false)  => { changes.push(Field::Stop); },      
                    _                            => { },
                };


                if changes.len() > 0 {
                    return Ok(handle.response.needs_modification(request, &changes));
                } else {
                    return Ok(handle.response.is_matched(request));
                }

            },

            TaskRequestType::Modify => {

                if request.changes.contains(&Field::Start)        { self.do_start(handle, request)?;   }
                else if request.changes.contains(&Field::Stop)    { self.do_stop(handle, request)?;    }
                else if request.changes.contains(&Field::Restart) { self.do_restart(handle, request)?; }

                if request.changes.contains(&Field::Enable)       { self.do_enable(handle, request)?;  }
                else if request.changes.contains(&Field::Disable) { self.do_disable(handle, request)?; }

                return Ok(handle.response.is_modified(request, request.changes.clone()));
            }
    
            _ => { return Err(handle.response.not_supported(request)); }
    
        }
    }

}

impl SystemdServiceAction {

    pub fn get_service_details(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<ServiceDetails,Arc<TaskResponse>> {
        
        let is_enabled : bool;
        let is_active  : bool;
        let is_enabled_cmd = format!("systemctl is-enabled '{}'", self.service);
        let is_active_cmd = format!("systemctl is-active '{}'", self.service);
        
        let result = handle.remote.run(request, &is_enabled_cmd, CheckRc::Unchecked)?;
        let (_rc,out) = cmd_info(&result);
        if out.find("disabled").is_some() || out.find("deactivating").is_some() { is_enabled = false; }
        else if out.find("enabled").is_some() || out.find("alias").is_some() { is_enabled = true; } 
        else {
            return Err(handle.response.is_failed(request, &format!("systemctl status unexpected for service({}): {}", self.service, out))); 
        }

        let result2 = handle.remote.run(request, &is_active_cmd, CheckRc::Unchecked)?;
        let (_rc2,out2) = cmd_info(&result2);
        if out2.find("inactive").is_some() { is_active = false; }
        else if out2.find("active").is_some() { is_active = true; }
        else { 
            return Err(handle.response.is_failed(request, &format!("systemctl status unexpected for service({}): {}", self.service, out))); 
        }

        return Ok(ServiceDetails {
            enabled: is_enabled,
            started: is_active,
        });
    }

    pub fn do_start(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("systemctl start '{}'", self.service);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }
    
    pub fn do_stop(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("systemctl stop '{}'", self.service);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }
    
    pub fn do_enable(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("systemctl enable '{}'", self.service);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }
    
    pub fn do_disable(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("systemctl disable '{}'", self.service);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

    pub fn do_restart(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>,Arc<TaskResponse>> {
        let cmd = format!("systemctl restart '{}'", self.service);
        return handle.remote.run(request, &cmd, CheckRc::Checked);
    }

}
