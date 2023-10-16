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

const MODULE: &str = "facts";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct FactsTask {
    pub name: Option<String>,
    pub facter: Option<String>,
    pub ohai: Option<String>,
    pub with: Option<PreLogicInput>,
    pub and: Option<PostLogicInput>
}
struct FactsAction {
    facter: bool,
    ohai: bool,
}

impl IsTask for FactsTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(FactsAction {
                    facter:  handle.template.boolean_option_default_false(&request, tm, &String::from("facter"), &self.facter)?,
                    ohai:    handle.template.boolean_option_default_false(&request, tm, &String::from("ohai"), &self.ohai)?,

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
            Some(HostOSType::AIX)     => { self.do_aix_facts(handle, request, &facts)?     },
            Some(HostOSType::HPUX)    => { self.do_hpux_facts(handle, request, &facts)?    },
            Some(HostOSType::Linux)   => { self.do_linux_facts(handle, request, &facts)?   },
            Some(HostOSType::MacOS)   => { self.do_mac_facts(handle, request, &facts)?     },
            Some(HostOSType::NetBSD)  => { self.do_netbsd_facts(handle, request, &facts)?  },
            Some(HostOSType::OpenBSD) => { self.do_openbsd_facts(handle, request, &facts)? },
            None => { return Err(handle.response.is_failed(request, &String::from("facts not implemented for OS Type"))) }
        };
        self.do_arch(handle, request, &facts)?;
        if self.facter {
            self.do_facter(handle, request, &facts)?;
        }
        if self.ohai {
            self.do_ohai(handle, request, &facts)?;

        }
        handle.host.write().unwrap().update_facts(&facts);
        return Ok(());
    }

    fn insert_string(&self, mapping: &Arc<RwLock<serde_yaml::Mapping>>, key: &String, value: &String) {
        mapping.write().unwrap().insert(serde_yaml::Value::String(key.clone()), serde_yaml::Value::String(value.clone())); 
    }

    fn insert_json(&self, mapping: &Arc<RwLock<serde_yaml::Mapping>>, key: &String, value: &String) -> Result<(), String> {
        match serde_json::from_str(value) {
            Ok(json) => { mapping.write().unwrap().insert(serde_yaml::Value::String(key.clone()), json); Ok(()) }
            Err(y) => Err(format!("error processing fact JSON: {:?}", y))
        }
    }

    fn do_aix_facts(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("UNIX"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("AIX"));
        self.do_aix_os_release(handle, request, mapping)?;
        return Ok(());
     }

    fn do_aix_os_release(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        // mimics the os_release variables even /etc/os-release does not exist
        let cmd = String::from("oslevel -s");
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        self.insert_string(mapping, &String::from("jet_os_release_version_id"), &out);
        return Ok(());
    }

    fn do_hpux_facts(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("UNIX"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("HP-UX"));
        self.do_hpux_os_release(handle, request, mapping)?;
        return Ok(());
    }

    fn do_mac_facts(&self, _handle: &Arc<TaskHandle>, _request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("MacOS"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("OSX"));
        return Ok(());
    }

    fn do_hpux_os_release(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        // mimics the os_release variables even /etc/os-release does not exist
        let cmd = String::from("uname -rv");
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        self.insert_string(mapping, &String::from("jet_os_release_version_id"), &out);
        return Ok(());
    }

    fn do_linux_facts(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("Linux"));
        self.do_linux_os_release(handle, request, mapping)?;
        return Ok(());
    }

    fn do_openbsd_facts(&self, _handle: &Arc<TaskHandle>, _request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("OpenBSD"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("OpenBSD"));
        return Ok(());
    }

    fn do_netbsd_facts(&self, _handle: &Arc<TaskHandle>, _request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        self.insert_string(mapping, &String::from("jet_os_type"), &String::from("NetBSD"));
        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("NetBSD"));
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
                    } else if v1.find("arch").is_some() {
                        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("Arch"))
                    }
                }
                // if /etc/os-release does not have ID_LIKE line, like Archlinux
                if k1.eq("id") {
                    if v1.find("arch").is_some() {
                        self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("Arch"));
                    }
                }
            }
        }
        // jet_os_flavor should always have a value to prevent errors from invalid templates
        if ! mapping.read().unwrap().contains_key("jet_os_flavor") {
            self.insert_string(mapping, &String::from("jet_os_flavor"), &String::from("Unknown"))
        }
        return Ok(());
    }

    fn do_arch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        let os_type = handle.host.read().unwrap().os_type.expect("os type");
        let cmd = match crate::tasks::cmd_library::get_arch_command(os_type) {
            Ok(x) => x,
            Err(_) => { return Err(handle.response.is_failed(request, &format!("unable to determine arch command for {:?}", os_type))) },
        };
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        self.insert_string(mapping, &String::from("jet_arch"), &String::from(out));
        return Ok(());
    }

    fn do_facter(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        let result = handle.remote.run(request, &String::from("facter --json"), CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        match self.insert_json(mapping, &String::from("facter"), &String::from(out)) {
            Ok(_) => {},
            Err(_) => { return Err(handle.response.is_failed(request, &String::from("failed to parse facter output"))) }
        }
        return Ok(());    }

    fn do_ohai(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, mapping: &Arc<RwLock<serde_yaml::Mapping>>) -> Result<(), Arc<TaskResponse>> {
        let result = handle.remote.run(request, &String::from("ohai"), CheckRc::Checked)?;
        let (_rc, out) = cmd_info(&result);
        match self.insert_json(mapping, &String::from("ohai"), &String::from(out)) {
            Ok(_) => {},
            Err(_) => { return Err(handle.response.is_failed(request, &String::from("failed to parse ohai output"))) }
        }
        return Ok(());
    }

}

