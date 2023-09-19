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

use std::sync::{Arc,RwLock};
use std::path::PathBuf;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use crate::inventory::hosts::Host;
use crate::playbooks::traversal::RunState;
use crate::playbooks::context::PlaybookContext;
use crate::tasks::cmd_library::{screen_path,screen_general_input_strict};
use crate::handle::response::Response;
use crate::playbooks::templar::{Templar,TemplateMode};

// template contains support code for all variable evaluation in the playbook language, as well as
// support for the template module, and ALSO the code to validate and process module arguments to make
// sure they are the right type.
//
// because module arguments come in as strings, we evaluate templates here and then see if they can
// be parsed as their desired types.

// when blend target must be specified, it is either the template module or *not*.
// the only real difference (at the moment) is that the template module is allowed access
// to environment variables which are prefixed as ENV_foo. The environment mechanism is how
// we work with secret manager tools. See the website secrets documentation for details

#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum BlendTarget {
    NotTemplateModule,
    TemplateModule,
}

// where used, safe means screening commands or arguments for unexpected shell characters
// that could lead to command escapes. Because a command is marked unsafe does not mean
// it is actually unsafe, it just means that it is not checked. A command using
// variables from untrusted sources may actually be unsafe, for instance, the shell
// module when used with 'unsafe: true'.  Though if no variables are used, it would
// be quite safe.

#[derive(Eq,Hash,PartialEq,Clone,Copy,Debug)]
pub enum Safety {
    Safe,
    Unsafe
}

pub struct Template {
    run_state: Arc<RunState>, 
    host: Arc<RwLock<Host>>, 
    response: Arc<Response>,
    detached_templar: Templar
}

impl Template {

    // templating is always done in reference to a specific host, so that we can mix in host specific variables
    // the response is in the constructor as need it to return errors that are passed upwards from
    // functions below.

    pub fn new(run_state_handle: Arc<RunState>, host_handle: Arc<RwLock<Host>>, response:Arc<Response>) -> Self {
        Self {
            run_state: run_state_handle,
            host: host_handle,
            response: response,
            detached_templar: Templar::new()
        }
    }

    pub fn get_context(&self) -> Arc<RwLock<PlaybookContext>> {
        return Arc::clone(&self.run_state.context);
    }

    fn unwrap_string_result(&self, request: &Arc<TaskRequest>, str_result: &Result<String,String>) -> Result<String, Arc<TaskResponse>> {
        return match str_result {
            Ok(x) => Ok(x.clone()),
            Err(y) => {
                return Err(self.response.is_failed(request, &y.clone()));
            }
        };
    }

    fn template_unsafe_internal(&self, request: &Arc<TaskRequest>, tm: TemplateMode, _field: &String, template: &String, blend_target: BlendTarget) -> Result<String,Arc<TaskResponse>> {
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host, blend_target, tm);
        if result.is_ok() {
            let result_ok = result.as_ref().unwrap();
            if result_ok.eq("") {
                return Err(self.response.is_failed(request, &format!("evaluated to empty string")));
            }
        }
        let result2 = self.unwrap_string_result(request, &result)?;
        return Ok(result2);
    }
    
    pub fn string_for_template_module_use_only(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // this is the version of templating that gives access to secret variables, we don't allow them elsewhere as they would be easy to leak to CI/CD/build output/logs
        // and the contents to templates are not shown to anything
        return self.template_unsafe_internal(request, tm, field, template, BlendTarget::TemplateModule);
    }

    pub fn string_unsafe_for_shell(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // indicates templating a string that will not without further processing, be passed to a shell command
        return self.template_unsafe_internal(request, tm, field, template, BlendTarget::NotTemplateModule);
    }


    // FIXME: this code is possibly a bit redundant - perhaps calling methods can use the public function and this can be eliminated

    fn string_option_unsafe(&self, request: &Arc<TaskRequest>, tm: TemplateMode,field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // templates a string that is not allowed to be used in shell commands and may contain special characters
        if template.is_none() { return Ok(None); }
        let result = self.string(request, tm, field, &template.as_ref().unwrap());
        return match result { 
            Ok(x) => Ok(Some(x)), 
            Err(y) => { Err(self.response.is_failed(request, &format!("field ({}) template error: {:?}", field, y))) } 
        };
    }
    
    pub fn string_option_unsafe_for_shell(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // indicates templating a string that will not without further processing, be passed to a shell command
        return match template.is_none() {
            true => Ok(None),
            false => Ok(Some(self.template_unsafe_internal(request, tm, field, &template.as_ref().unwrap(), BlendTarget::NotTemplateModule)?))
        }
    }

    pub fn string(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // templates a required string parameter - the simplest of argument processing, this requires no casting to other types
        let result = self.string_unsafe_for_shell(request, tm, field, template);
        return match result {
            Ok(x) => match screen_general_input_strict(&x) {
                Ok(y) => Ok(y),
                Err(z) => { return Err(self.response.is_failed(request, &format!("field {}, {}", field, z))) }
            },
            Err(y) => Err(y)
        };
    }

    pub fn string_no_spaces(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // same as self.string above, this version also does not allow spaces in the resulting string
        let value = self.string(request, tm, field, template)?;
        if self.has_spaces(&value) {
            return Err(self.response.is_failed(request, &format!("field ({}): spaces are not allowed", field)))
        }
        return Ok(value.clone());
    }

    pub fn string_option_no_spaces(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // this is a version of string_no_spaces that allows the value to be optional
        let prelim = self.string_option(request, tm, field, template)?;
        if prelim.is_some() {
            let value = prelim.as_ref().unwrap();
            if self.has_spaces(&value) {
                return Err(self.response.is_failed(request, &format!("field ({}): spaces are not allowed", field)))
            }
        }
        return Ok(prelim.clone());
    }

    pub fn string_option_trim(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // for processing parameters that take optional strings, but make sure to remove any extra surrounding whitespace
        // YAML should do this anyway so it's mostly overkill but may prevent some rare errors from inventory variable sources
        let prelim = self.string_option(request, tm, field, template)?;
        if prelim.is_some() {
            return Ok(Some(prelim.unwrap().trim().to_string()));
        }
        return Ok(None);
    }

    pub fn no_template_string_option_trim(&self, input: &Option<String>) -> Option<String> {
        // takes a string option and uses it verbatim, for parameters that do not allow variables in them
        if input.is_some() {
            let value = input.as_ref().unwrap();
            return Some(value.trim().to_string());
        }
        return None;
    }

    pub fn path(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<String,Arc<TaskResponse>> {
        // templates a string and makes sure the output looks like a valid path
        let result = self.run_state.context.read().unwrap().render_template(template, &self.host, BlendTarget::NotTemplateModule, tm);
        let result2 = self.unwrap_string_result(request, &result)?;
        return match screen_path(&result2) {
            Ok(x) => Ok(x), Err(y) => { return Err(self.response.is_failed(request, &format!("{}, for field {}", y, field))) }
        }
    }



    pub fn string_option(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>) -> Result<Option<String>,Arc<TaskResponse>> {
        // templates an optional string
        let result = self.string_option_unsafe(request, tm, field, template);
        return match result {
            Ok(x1) => match x1 {
                Some(x) => match screen_general_input_strict(&x) {
                    Ok(y) => Ok(Some(y)),
                    Err(z) => { return Err(self.response.is_failed(request, &format!("field {}, {}", field, z))) }
                },
                None => Ok(None)
            },
            Err(y) => Err(y)
        };
    }

    #[allow(dead_code)]
    pub fn integer(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String)-> Result<u64,Arc<TaskResponse>> {
        // templates a required value that must resolve to an integer
        if tm == TemplateMode::Off {
            return Ok(0);
        }
        let st = self.string(request, tm, field, template)?;
        let num = st.parse::<u64>();
        return match num {
            Ok(num) => Ok(num), 
            Err(_err) => Err(self.response.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    pub fn integer_option(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>, default: u64) -> Result<u64,Arc<TaskResponse>> {
        // templates an optional value that must resolve to an integer
        if tm == TemplateMode::Off {
            return Ok(0);
        }
        if template.is_none() {
            return Ok(default); 
        }
        let st = self.string(request, tm, field, &template.as_ref().unwrap())?;
        let num = st.parse::<u64>();
        // FIXME: these can use map_err
        return match num {
            Ok(num) => Ok(num), 
            Err(_err) => Err(self.response.is_failed(request, &format!("field ({}) value is not an integer: {}", field, st)))
        }
    }

    #[allow(dead_code)]
    pub fn boolean(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &String) -> Result<bool,Arc<TaskResponse>> {
        // templates a required value that must resolve to a boolean
        // where possible, consider using boolean_option_default_true/false instead
        // jet mostly favors booleans defaulting to false, but it doesn't always make sense
        if tm == TemplateMode::Off {
            return Ok(true);
        }
        let st = self.string(request, tm, field, template)?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x), Err(_err) => Err(self.response.is_failed(request, &format!("field ({}) value is not a boolean: {}", field, st)))
        }
    }

    #[allow(dead_code)]
    pub fn boolean_option_default_true(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        // templates an optional value that resolves to a boolean, if omitted, assume the answer is true
        return self.internal_boolean_option(request, tm, field, template, true);
    }

    pub fn boolean_option_default_false(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>)-> Result<bool,Arc<TaskResponse>>{
        // templates an optional value that resolves to a boolean, if omitted, assume the answer is false
        return self.internal_boolean_option(request, tm, field, template, false);
    }
  
    fn internal_boolean_option(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>, default: bool)-> Result<bool,Arc<TaskResponse>>{
        // supporting code for boolean parsing above
        if tm == TemplateMode::Off {
            return Ok(false);
        }
        if template.is_none() {
            return Ok(default);
        }
        let st = self.string(request, tm, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(x),
            Err(_err) => Err(self.response.is_failed(request, &format!("field ({}) value is not a boolean: {}", field, st)))
        }
    }

    pub fn boolean_option_default_none(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, template: &Option<String>)-> Result<Option<bool>,Arc<TaskResponse>>{
        // supports an optional boolean value that does not default to true or false - effectively making the option a trinary value where None is "no preference"
        if tm == TemplateMode::Off {
            return Ok(None);
        }
        if template.is_none() {
            return Ok(None);
        }
        let st = self.string(request, tm, field, &template.as_ref().unwrap())?;
        let x = st.parse::<bool>();
        return match x {
            Ok(x) => Ok(Some(x)), 
            Err(_err) => Err(self.response.is_failed(request, &format!("field ({}) value is not a boolean: {}", field, st)))
        }
    }

    pub fn test_condition(&self, request: &Arc<TaskRequest>, tm: TemplateMode, expr: &String) -> Result<bool, Arc<TaskResponse>> {
        // used to evaluate in-language conditionals throughout the program.
        if tm == TemplateMode::Off {
            return Ok(false);
        }
        let result = self.get_context().read().unwrap().test_condition(expr, &self.host, tm);
        return match result {
            Ok(x) => Ok(x), Err(y) => Err(self.response.is_failed(request, &y))
        }
    }

    pub fn test_condition_with_extra_data(&self, request: &Arc<TaskRequest>, tm: TemplateMode, expr: &String, _host: &Arc<RwLock<Host>>, vars_input: serde_yaml::Mapping) -> Result<bool,Arc<TaskResponse>> {
        // same as test_condition but mixes in some temporary data that is not stored elsewhere for future template evaluation
        if tm == TemplateMode::Off {
            return Ok(false);
        }
        let result = self.get_context().read().unwrap().test_condition_with_extra_data(expr, &self.host, vars_input, tm);
        return match result {
            Ok(x) => Ok(x), Err(y) => Err(self.response.is_failed(request, &y))
        }
    }

    pub fn find_template_path(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        // templates a string and then looks for the resulting file in the logical templates/ locations (if not an absolute path)
        // raises errors if the source files are not found
        return self.find_sub_path(&String::from("templates"), request, tm, field, str_path);
    }

    pub fn find_file_path(&self, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        // simialr to find_template_path, this one assumes a 'files/' directory for relative paths.
        return self.find_sub_path(&String::from("files"), request, tm, field, str_path);
    }

    fn find_sub_path(&self, prefix: &String, request: &Arc<TaskRequest>, tm: TemplateMode, field: &String, str_path: &String) -> Result<PathBuf, Arc<TaskResponse>> {
        // supporting code for find_template_path and find_file_path
        if tm == TemplateMode::Off {
            return Ok(PathBuf::new());
        }
        let prelim = match screen_path(&str_path) {
            Ok(x) => x, 
            Err(y) => { return Err(self.response.is_failed(request, &format!("{}, for field: {}", y, field))) }
        };
        let mut path = PathBuf::new();
        path.push(prelim);
        if path.is_absolute() {
            if path.is_file() {
                return Ok(path);
            } else {
                return Err(self.response.is_failed(request, &format!("field ({}): no such file: {}", field, str_path)));
            }
        } else {
            let mut path2 = PathBuf::new();
            path2.push(prefix);
            path2.push(str_path);
            if path2.is_file() {
                return Ok(path2);
            } else {
                return Err(self.response.is_failed(request, &format!("field ({}): no such file: {}", field, str_path)));
            }
        }
    }

    fn has_spaces(&self, input: &String) -> bool {
        let found = input.find(" ");
        return found.is_some();
    }

    pub fn add_sudo_details(&self, request: &TaskRequest, cmd: &String) -> Result<String, String> {
        // this is used by remote.rs to modify any command, inserting the results of evaluating the configured sudo_template
        // instead of the original command. only specific variables are allowed in the sudo template as opposed
        // to all the variables in jet's current host context.
        if ! request.is_sudoing() {
            return Ok(cmd.clone());
        }
        let details = request.sudo_details.as_ref().unwrap();
        let user = details.user.as_ref().unwrap().clone();
        let sudo_template = details.template.clone();
        let mut data = serde_yaml::Mapping::new();            
        data.insert(serde_yaml::Value::String(String::from("jet_sudo_user")), serde_yaml::Value::String(user.clone()));
        data.insert(serde_yaml::Value::String(String::from("jet_command")), serde_yaml::Value::String(cmd.clone()));
        let result = self.detached_templar.render(&sudo_template, data, TemplateMode::Strict)?;
        return Ok(result)
    }


}