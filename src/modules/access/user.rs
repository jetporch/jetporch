// Jetporch
// Copyright (C) 2023 - JetPorch Project Contributors
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

use crate::inventory::hosts::HostOSType;
use crate::tasks::*;
use crate::handle::handle::TaskHandle;
use crate::tasks::fields::Field;
use serde::{Deserialize};
use std::collections::{HashSet};
use std::sync::Arc;
use std::vec::Vec;

const MODULE: &str = "user";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct UserTask {
    pub name:           Option<String>,
    pub user:           String,
    pub uid:            Option<String>,
    pub gid:            Option<String>,
    pub groups:         Option<HashSet<String>>,
    pub append:         Option<String>,
    pub create_home:    Option<String>,
    pub gecos:          Option<String>,
    pub shell:          Option<String>,
    pub remove:         Option<String>,
    pub cleanup:        Option<String>, // remove $HOME and $MAIL
    pub with:           Option<PreLogicInput>,
    pub and:            Option<PostLogicInput>
}

struct UserAction {
    pub user:           String,
    pub uid:            Option<u64>,
    pub gid:            Option<String>,
    pub groups:         Option<HashSet<String>>,
    pub append:         bool,
    pub create_home:    bool,
    pub gecos:          Option<String>,
    pub shell:          Option<String>,
    pub remove:         bool,
    pub cleanup:        bool,
}

struct UserDetails {
    exists:     bool,
    gid:        Option<String>,
    groups:     Option<HashSet<String>>,
    gecos:      Option<String>,
    shell:      Option<String>,
}

impl IsTask for UserTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {

        return Ok(
            EvaluatedTask {
                action: Arc::new(UserAction {
                    user:           handle.template.string_no_spaces(request, tm, &String::from("user"), &self.user)?,
                    uid:            handle.template.integer_option(request, tm, &String::from("uid"), &self.uid, None)?,
                    gid:            handle.template.string_option(request, tm, &String::from("gid"), &self.gid)?,
                    groups:         {
                        match &self.groups {
                            Some(groups) => {
                                let mut templated_groups: HashSet<String> = HashSet::new();
                                 for group in groups {
                                    templated_groups.insert(handle.template.string_no_spaces(request, tm, &String::from("groups"), group)?);
                                 }
                                 Some(templated_groups)
                                }
                            None => {None}
                        }
                    },
                    append:         handle.template.boolean_option_default_false(&request, tm, &String::from("append"), &self.append)?,
                    create_home:    handle.template.boolean_option_default_false(&request, tm, &String::from("create_home"), &self.create_home)?,
                    gecos:          handle.template.string_option(request, tm, &String::from("gecos"), &self.gecos)?,
                    shell:          handle.template.string_option(request, tm, &String::from("shell"), &self.shell)?,
                    remove:         handle.template.boolean_option_default_false(&request, tm, &String::from("remove"), &self.remove)?,
                    cleanup:        handle.template.boolean_option_default_false(&request, tm, &String::from("cleanup"), &self.cleanup)?,
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for UserAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        let os_type = handle.host.read().unwrap().os_type.unwrap();
        let actual: UserDetails = self.get_user_details(handle, request)?;

        match request.request_type {

            TaskRequestType::Query => {

                match (actual.exists, self.remove) {
                    (false, true)  => return Ok(handle.response.is_matched(request)),
                    (false, false) => return Ok(handle.response.needs_creation(request)),
                    (true, true)   => return Ok(handle.response.needs_removal(request)),
                    (true, false)  => {

                        let mut changes : Vec<Field> = Vec::new();
                        if UserAction::string_wants_change(&self.gid, &actual.gid) { changes.push(Field::Gid); }
                        if UserAction::string_wants_change(&self.gecos, &actual.gecos) { changes.push(Field::Gecos); }
                        if UserAction::string_wants_change(&self.shell, &actual.shell){ changes.push(Field::Shell); }
                        if self.groups_wants_change(&actual) { changes.push(Field::Groups); }

                        match changes.len() {
                            0 => return Ok(handle.response.is_matched(request)),
                            _ => return Ok(handle.response.needs_modification(request, &changes)),
                        }
                    }
                }
            },

            TaskRequestType::Create => {
                let cmd_result = self.create_user_command(os_type);
                let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
                handle.remote.run(request, &cmd, CheckRc::Checked)?;
                return Ok(handle.response.is_created(request));
            },

            TaskRequestType::Modify => {
                let cmd_result = self.modify_user_command(os_type, &actual);
                let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
                handle.remote.run(request, &cmd, CheckRc::Checked)?;
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            },

            TaskRequestType::Remove => {
                let cmd_result = self.delete_user_command(os_type);
                let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
                handle.remote.run(request, &cmd, CheckRc::Checked)?;
                return Ok(handle.response.is_removed(request))
            }

            // no passive or execute leg
            _ => { return Err(handle.response.not_supported(request)); }


        }
    }
}

impl UserAction {

    fn get_gid(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<String,Arc<TaskResponse>>  {
        let os_type = handle.host.read().unwrap().os_type.unwrap();
        let cmd_result = self.get_user_gid_command(os_type);
        let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_, out) = cmd_info(&result);
        return Ok(out);
    }

    fn get_groups(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<HashSet<String>,Arc<TaskResponse>>  {
        let os_type = handle.host.read().unwrap().os_type.unwrap();
        let cmd_result = self.get_user_groups_command(os_type);
        let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
        let result = handle.remote.run(request, &cmd, CheckRc::Checked)?;
        let (_, out) = cmd_info(&result);
        let str_vec: Vec<&str> = out.split_whitespace().collect();
        let groups: HashSet<String> = str_vec.iter().map(|&s| s.to_string()).collect();
        return Ok(groups);
    }

    fn get_user_details (&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<UserDetails,Arc<TaskResponse>> {
        let os_type = handle.host.read().unwrap().os_type.unwrap();
        let cmd_result = self.get_user_command(os_type);
        let cmd = handle.remote.unwrap_string_result(request, &cmd_result)?;
        let result = handle.remote.run(request, &cmd, CheckRc::Unchecked)?;
        let (rc, out) = cmd_info(&result);

        match rc {
            // return early if user does not exist (rc = 2)
            2 => {
                return Ok(UserDetails {
                    exists:     false,
                    gid:        None,
                    groups:     None,
                    gecos:      None,
                    shell:      None,
                })
            }
            0 => {
                let items: Vec<&str> = out.split(":").collect();
                let gid =  Some(self.get_gid(handle, request)?);
                let groups = Some(self.get_groups(handle, request)?);
                    return Ok(UserDetails {
                        exists: true,
                        gid:    gid,
                        groups: groups,
                        gecos:  Some(items[4].to_string()),
                        shell:  Some(items[6].to_string()),
                    })
            }
            error => { return Err(handle.response.is_failed(request, &format!("failure getting user details, rc: '{}'",error))); }
        }
    }

    fn get_user_command(&self, os_type: HostOSType) -> Result<String,String> {
        // returns a string devided by 6 colons (':')
        // user:pwd:UID:GID:Gecos:Homedir:Shell
        // F.e.: alice:x:1000:1000:alice:/home/alice:/bin/bash
        // Of course: the pwd field does just contain an 'x' in modern Unix/Linux because of /etc/shadow
        match os_type {
            HostOSType::Linux => { return Ok(format!("getent passwd '{}'", self.user)); },
            HostOSType::MacOS => { return Err(format!("user module not yet supported for '{:?}'", os_type)) },
        }
    }

    fn create_user_command(&self, os_type: HostOSType) -> Result<String,String> {
        let mut cmd = String::new();
        match os_type {
            HostOSType::Linux => cmd.push_str("useradd"),
            HostOSType::MacOS => return Err(format!("user module not yet supported for '{:?}'", os_type)),
        }
        if self.uid.is_some() {
            cmd.push_str(&format!(" -u '{}'", self.uid.as_ref().unwrap()));
        }
        if self.gid.is_some() {
            cmd.push_str(&format!(" -g '{}'", self.gid.as_ref().unwrap()));
        }
        if self.groups.is_some() {
            let final_groups: Vec<String> = self.groups.as_ref().unwrap().iter().cloned().collect();
            cmd.push_str(&format!(" -G '{}'", final_groups.join(",")));
        }
        if self.create_home {
            cmd.push_str(" -m");
        }
        if self.gecos.is_some() {
            cmd.push_str(&format!(" -c '{}'", self.gecos.as_ref().unwrap()));
        }
        if self.shell.is_some() {
            cmd.push_str(&format!(" -s '{}'", self.shell.as_ref().unwrap()));
        }
        cmd.push_str(&format!(" '{}'", self.user));

        return Ok(cmd);
    }

    fn modify_user_command(&self, os_type: HostOSType, actual: &UserDetails) -> Result<String,String> {
        let mut cmd = String::new();

        match os_type {
            HostOSType::Linux => cmd.push_str("usermod"),
            HostOSType::MacOS => return Err(format!("user module not yet supported for '{:?}'", os_type)),
        }
        if self.gid.is_some() {
            cmd.push_str(&format!(" -g '{}'", self.gid.as_ref().unwrap()));
        }
        if self.gecos.is_some() {
            cmd.push_str(&format!(" -c '{}'", self.gecos.as_ref().unwrap()));
        }
        if self.shell.is_some() {
            cmd.push_str(&format!(" -s '{}'", self.shell.as_ref().unwrap()));
        }

        if self.groups.is_some() {
            match self.append {
                    true => {
                        match &actual.groups {
                            // if some groups already exist, we need to add the new ones
                            Some(actual_groups) => {
                                let mut groups = self.groups.clone().unwrap();
                                for group in actual_groups {
                                    groups.insert(group.clone());
                                }
                                let final_groups: Vec<String> = groups.iter().cloned().collect();
                                cmd.push_str(&format!(" -G '{}'",final_groups.join(",")));
                            }
                            // otherwise we just take the new ones
                            None => {
                                let final_groups: Vec<String> = self.groups.as_ref().unwrap().iter().cloned().collect();
                                cmd.push_str(&format!(" -G '{}'",final_groups.join(",")));
                            }
                        }
                    }
                    // just replace existing groups with new groups
                    false => {
                        let final_groups: Vec<String> = self.groups.as_ref().unwrap().iter().cloned().collect();
                        cmd.push_str(&format!(" -G '{}'", final_groups.join(",")));
                    }
            }
        }
        cmd.push_str(&format!(" '{}'", self.user));

        return Ok(cmd);
    }

    fn delete_user_command(&self, os_type: HostOSType) -> Result<String,String> {
        match os_type {
            HostOSType::Linux => {
                match self.cleanup {
                    false => return Ok(format!("userdel '{}'", self.user)),
                    true => return Ok(format!("userdel -r '{}'", self.user)),
                }
            },
            HostOSType::MacOS => return Err(format!("user module not yet supported for '{:?}'", os_type)),
        }
    }

    fn get_user_gid_command(&self, os_type: HostOSType) -> Result<String,String> {
        // Returns a string containing the primary group name.
        match os_type {
            HostOSType::Linux => { return Ok(format!("id -gn '{}'", self.user)); },
            HostOSType::MacOS => return Err(format!("user module not yet supported for '{:?}'", os_type)),
        }
    }

    fn get_user_groups_command(&self, os_type: HostOSType) -> Result<String,String> {
        // Returns a string containing a space separated list of group names.
          match os_type {
            HostOSType::Linux => { return Ok(format!("id -Gn '{}'", self.user)); },
            HostOSType::MacOS => return Err(format!("user module not yet supported for '{:?}'", os_type)),
        }
    }

    fn string_wants_change(our: &Option<String>, actual: &Option<String>) -> bool {
        // we only act if there is a value defined in yaml (our):
        if our.is_some() {
            match actual {
                // we only need to compare if the host (actual) has that value set:
                Some(actual_value) => if ! our.as_ref().unwrap().eq(actual_value) {
                    // we only need to change if both values differ -> change:
                    return true;
                }
                // Otherwise we create that value -> change
                None => { return true; }
            }
        }
        // otherwise no change
        return false;
    }

    fn groups_wants_change(&self, actual: &UserDetails) -> bool {
        // we only act if there is a value defined in yaml (self.groups):
        if self.groups.is_some() {
            match &actual.groups {
                // we only need to compare if the host (actual) has groups:
                Some(actual_groups) => {
                    // to compare correct we need to add the actual primary group
                    // to our defined groups and compare them.
                    let actual_gid =  actual.gid.as_ref().unwrap();
                    let mut groups = self.groups.clone().unwrap();
                    groups.insert(actual_gid.to_string());
                    // compare depends on append mode
                    match &self.append {
                        true => {
                            if ! (groups.is_subset(&actual_groups)) {
                                return true;
                            }
                        }
                        false => {
                            if ! (&groups == actual_groups) {
                                return true;
                            }
                        }
                    }
                }
                // Otherwise we create that groups -> change
                None => { return true; }
            }
        }
        // otherwise no change
        return false;
    }

}
