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

const MODULE: &str = "group";

#[derive(Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct GroupTask {
    pub name:           Option<String>,
    pub group:          String,
    pub gid:            Option<String>,
    pub users:          Option<HashSet<String>>,
    pub append:         Option<String>,
    pub system:         Option<String>,
    pub remove:         Option<String>,
    pub with:           Option<PreLogicInput>,
    pub and:            Option<PostLogicInput>
}

struct GroupAction {
    pub group:          String,
    pub gid:            Option<u64>,
    pub users:          Option<HashSet<String>>,
    pub append:         bool,
    pub system:         bool,
    pub remove:         bool,
}

struct GroupDetails {
    exists:     bool,
    gid:        Option<u64>,
    users:      Option<HashSet<String>>,
}

impl IsTask for GroupTask {

    fn get_module(&self) -> String { String::from(MODULE) }
    fn get_name(&self) -> Option<String> { self.name.clone() }
    fn get_with(&self) -> Option<PreLogicInput> { self.with.clone() }

    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>> {
        return Ok(
            EvaluatedTask {
                action: Arc::new(GroupAction {
                    group:          handle.template.string_no_spaces(request, tm, &String::from("group"), &self.group)?,
                    gid:            handle.template.integer_option(request, tm, &String::from("gid"), &self.gid, None)?,
                    users:          {
                        match &self.users {
                            Some(users) => {
                                let mut templated_users: HashSet<String> = HashSet::new();
                                for user in users {
                                    templated_users.insert(handle.template.string_no_spaces(request, tm, &String::from("users"), user)?);
                                }
                                Some(templated_users)
                            },
                            None => {None}
                        }
                    },
                    append:         handle.template.boolean_option_default_false(&request, tm, &String::from("append"), &self.append)?,
                    system:         handle.template.boolean_option_default_false(&request, tm, &String::from("system"), &self.system)?,
                    remove:         handle.template.boolean_option_default_false(&request, tm, &String::from("remove"), &self.remove)?,
                }),
                with: Arc::new(PreLogicInput::template(&handle, &request, tm, &self.with)?),
                and: Arc::new(PostLogicInput::template(&handle, &request, tm, &self.and)?),
            }
        );
    }

}

impl IsAction for GroupAction {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>> {

        match request.request_type {

            TaskRequestType::Query => {
                let os_type = handle.host.read().unwrap().os_type.unwrap();
                if os_type != HostOSType::Linux {
                    return Err(handle.response.is_failed(request, &String::from("this group module only supports Linux")));
                }
                let actual: GroupDetails = self.get_group_details(handle, request)?;
                match (actual.exists, self.remove) {
                    (false, true)  => return Ok(handle.response.is_matched(request)),
                    (false, false) => return Ok(handle.response.needs_creation(request)),
                    (true, true)   => return Ok(handle.response.needs_removal(request)),
                    (true, false)  => {

                        let mut changes : Vec<Field> = Vec::new();
                        if GroupAction::u64_wants_change(&self.gid, &actual.gid) { changes.push(Field::Gid); }
                        if self.users_wants_change(&actual) { changes.push(Field::Users); }

                        match changes.len() {
                            0 => return Ok(handle.response.is_matched(request)),
                            _ => return Ok(handle.response.needs_modification(request, &changes)),
                        }
                    }
                }
            },

            TaskRequestType::Create => {
                let cmd = self.create_group_command();
                handle.remote.run(request, &cmd, CheckRc::Checked)?;
                if self.create_group_users_command().is_some() {
                    let cmd = self.create_group_users_command().unwrap();
                    handle.remote.run(request, &cmd, CheckRc::Checked)?;
                }
                return Ok(handle.response.is_created(request));
            },

            TaskRequestType::Modify => {
                let actual: GroupDetails = self.get_group_details(handle, request)?;
                if self.modify_group_command(&actual, &request.changes).is_some() {
                    let cmd = self.modify_group_command(&actual, &request.changes).unwrap();
                    handle.remote.run(request, &cmd, CheckRc::Checked)?;
                }
                if self.modify_group_users_command(&actual, &request.changes).is_some() {
                    let cmd = self.modify_group_users_command(&actual, &request.changes).unwrap();
                    handle.remote.run(request, &cmd, CheckRc::Checked)?;
                }
                return Ok(handle.response.is_modified(request, request.changes.clone()));
            },

            TaskRequestType::Remove => {
                let cmd = self.delete_group_command();
                handle.remote.run(request, &cmd, CheckRc::Checked)?;
                return Ok(handle.response.is_removed(request))
            }

            // no passive or execute leg
            _ => { return Err(handle.response.not_supported(request)); }

        }
    }
}

impl GroupAction {

    fn get_group_details (&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<GroupDetails,Arc<TaskResponse>> {
        let cmd = self.get_group_command();
        let result = handle.remote.run(request, &cmd, CheckRc::Unchecked)?;
        let (rc, out) = cmd_info(&result);

        match rc {
            // return early if group does not exist (rc = 2)
            2 => {
                return Ok(GroupDetails {
                    exists:   false,
                    gid:      None,
                    users:    None,
                })
            }
            0 => {
                let items: Vec<&str> = out.split(":").collect();
                let users: HashSet<String>;
                // getent group from pkg shadow on alpine leaves the third colon off
                // if no users are asigned to the group. F.e:
                // A group with users assigned on any linux including alpine
                //     users:x:100:alice,bob
                // A group with no users assigned on any linux
                //     users:x:100:
                // versus alpine
                //     users:x:100
                // Thus the length of  a Vec after split is always 4 on any linux exept alpine
                // where it is 3 if no users assigned. So we need to handle that here:
                if items.len() == 4 {
                    let str_vec: Vec<&str> = items[3].split(",").collect();
                    users = str_vec.iter().map(|&s| s.to_string()).collect();
                } else {
                    users = HashSet::new();
                }
                return Ok(GroupDetails {
                    exists: true,
                    gid:    Some(items[2].parse().unwrap()),
                    users: Some(users),
                })
            }
            x => { return Err(handle.response.is_failed(request, &format!("failure getting group details, rc: '{}'", x))); }
        }
    }

    fn get_group_command(&self) -> String {
        // returns a string devided by 3 colons (':')
        // group:pwd:GID:users
        // F.e.: users:x:100:alice,bob
        // Of course: the pwd field does just contain an 'x' in modern Unix/Linux because of /etc/shadow
        return format!("getent group '{}'", self.group);
    }

    fn create_group_command(&self) -> String {
        let mut cmd = String::from("groupadd");
        if self.gid.is_some() {
            cmd.push_str(&format!(" -g '{}'", self.gid.as_ref().unwrap()));
        }
        if self.system && self.gid.is_none() {
             cmd.push_str(&format!(" -r"));
        }
        cmd.push_str(&format!(" '{}'", self.group));
        return cmd;
    }

    fn create_group_users_command(&self) -> Option<String> {
        if self.users.is_some() {
            let final_users: Vec<String> = self.users.as_ref().unwrap().iter().cloned().collect();
            return Some(format!("gpasswd -M '{}' '{}'", final_users.join(","), self.group));
        } else {
            return None;
        }
    }

    fn modify_group_command(&self, _actual: &GroupDetails, changes: &Vec<Field>) -> Option<String> {
        if changes.contains(&Field::Gid) {
            let mut cmd = String::from("groupmod");
            if self.gid.is_some() && changes.contains(&Field::Gid) {
                cmd.push_str(&format!(" -g '{}'", self.gid.as_ref().unwrap()));
            }
            cmd.push_str(&format!(" '{}'", self.group));
            return Some(cmd);
        }
        return None;
    }

    fn modify_group_users_command(&self, actual: &GroupDetails, changes: &Vec<Field>) -> Option<String> {
        if self.users.is_some() && changes.contains(&Field::Users) {
            match self.append {
                true => {
                    match &actual.users {
                        // if some users already exist, we need to add the new ones
                        Some(actual_users) => {
                            let mut users = self.users.clone().unwrap();
                            for user in actual_users {
                                 users.insert(user.clone());
                            }
                            let final_users: Vec<String> = users.iter().cloned().collect();
                            return Some(format!("gpasswd -M '{}' '{}'",final_users.join(","), self.group))
                        },
                        // otherwise we just take the new ones
                        None => {
                            let final_users: Vec<String> = self.users.as_ref().unwrap().iter().cloned().collect();
                            return Some(format!("gpasswd -M '{}' '{}'",final_users.join(","), self.group))
                        }
                    }
                },
                // just replace existing users with new users
                false => {
                    let final_users: Vec<String> = self.users.as_ref().unwrap().iter().cloned().collect();
                    return Some(format!("gpasswd -M '{}' '{}'", final_users.join(","), self.group))
                }
            }
        }
        return None;
    }

    fn delete_group_command(&self) -> String {
        return format!("groupdel '{}'", self.group)
    }

    fn u64_wants_change(our: &Option<u64>, actual: &Option<u64>) -> bool {
        if our.is_some() {
            if actual.is_none() {
                return true
            }
            if ! our.as_ref().unwrap().eq(actual.as_ref().unwrap()) {
                return true;
            }
        }
        return false;
    }

    fn users_wants_change(&self, actual: &GroupDetails) -> bool {
        if self.users.is_none() {
            // no preference about configuration on the remote system
            return false
        }
        if actual.users.is_none() {
            // no remote users yet
            return true;
        }
        let actual_users  = actual.users.as_ref().unwrap();
        let desired_users = self.users.clone().unwrap();
        if self.append {
            return ! desired_users.is_subset(&actual_users);
        } else {
            return desired_users != *actual_users
        }
    }

}
