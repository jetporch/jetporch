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
use crate::handle::handle::TaskHandle;
use crate::inventory::hosts::{HostOSType};
//#[allow(unused_imports)]
use serde::Deserialize;
use std::sync::{Arc,RwLock};

const MODULE: &'static str = "facts";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FactsTask {
    pub name: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct FactsAction {
}

impl IsTask for FactsTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(FactsAction {
                }),
                with: Arc::new(PreLogicInput::template(handle, request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(handle, request, tm, &self.and)?),
            }
        );
    }
}

impl IsAction for FactsAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                return Ok(handle.response.needs_passive(request));
            },

            TaskRequestType::Passive => {
                self.do_facts(handle, request)?;
                return Ok(handle.response.is_passive(request));
            },

            _ => { return Err(handle.response.not_supported(request)); }

        }
    }

}

impl FactsAction {
    
    fn do_facts(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<(), Arc<TaskResponse>> {
        let os_type = handle.host.read().unwrap().os_type;
        let facts = Arc::new(RwLock::new(serde_yaml::Mapping::new()));
        match os_type {
            Some(HostOSType::Linux) => { self.do_linux_facts(handle, request, &facts)?; },
            Some(HostOSType::MacOS) => { self.do_mac_facts(handle, request, &facts)?;   }
            None => { return Err(handle.response.is_failed(request, &String::from("facts not implemented for OS Type"))); }
        };
        handle.host.write().unwrap().update_facts(&facts);
        return Ok(());
    }

    fn insert_string(&self, mapping: &Arc<RwLock<serde_yaml::Mapping>>, key: &String, value: &String) {
        mapping.write().unwrap().insert(serde_yaml::Value::String(key.clone()), serde_yaml::Value::String(value.clone())); 
    }

    fn do_mac_facts(&self, _handle: &Arc<TaskHandle>, _request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        // sets jet_os_type=MacOS
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("MacOS"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("OSX"));

        return Ok(());
    }

    fn do_linux_facts(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        // sets jet_os_type=Linux
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("Linux"));
        // and more facts...
        self.do_linux_os_release(handle, request, mapping)?;
        return Ok(());
    }

    fn do_linux_os_release(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        // makes a lot of variables from everything in /etc/os-release with a jet_os_release prefix such as:
        // jet_os_release_id="rocky" 
        // jet_os_release_platform_id="platform:el9"
        // jet_os_release_id_like="rhel centos fedora"
        // not all keys are available on all platforms 
        // more facts will be added from other sources later, some may be conditional based on distro
        let cmd = String::from("cat /etc/os-release");
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        for line in out.lines() {
            let mut tokens = line.split("=");
            let key = tokens.nth(0);
            let value = tokens.nth(0);
            if key.is_some() && value.is_some() {
                let mut k1 = key.unwrap().trim().to_string();
                k1.make_ascii_lowercase();
                let v1 = value.unwrap().trim().to_string().replace("\"","");
                self.insert_string(mapping, &format!("jet_os_release_{}", k1.to_string()), &v1.clone());
                if k1.eq("id_like") {
                    if v1.find("rhel").is_some() {
                        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("EL"));
                    } else if v1.find("debian").is_some() {
                        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("Debian"))
                    }
                }
            }
        }
        return Ok(());
    }
}

