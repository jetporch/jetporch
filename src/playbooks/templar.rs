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

use serde_yaml;
use once_cell::sync::Lazy;

use handlebars::{Handlebars,RenderError};

static HANDLEBARS: Lazy<Handlebars> = Lazy::new(|| {
    let mut hb = Handlebars::new();
    hb.register_escape_fn(handlebars::no_escape);
    hb.set_strict_mode(true);
    return hb;
});

static HANDLEBARS_UNSTRICT: Lazy<Handlebars> = Lazy::new(|| {
    let mut hb = Handlebars::new();
    hb.register_escape_fn(handlebars::no_escape);
    hb.set_strict_mode(true);
    return hb;
});

#[derive(PartialEq,Copy,Clone,Debug)]
pub enum TemplateMode {
    Strict,
    NotStrict,
    Off
}

pub struct Templar {
}

impl Templar {

    pub fn new() -> Self {
        return Self {
        };
    }

    pub fn render(&self, template: &String, data: serde_yaml::Mapping, template_mode: TemplateMode) -> Result<String, String> {
        let result : Result<String, RenderError> = match template_mode {
            TemplateMode::Strict => HANDLEBARS.render_template(template, &data),
            TemplateMode::NotStrict => HANDLEBARS_UNSTRICT.render_template(template, &data),
            /* this is only used to get back the raw 'items' collection inside the task FSM */
            TemplateMode::Off => Ok(String::from("empty"))
        };
        return match result {
            Ok(x) => {
                Ok(x)
            },
            Err(y) => {
                Err(format!("Template error: {}", y.desc))
            }
        }
    }

    pub fn render_value(&self, template: &String, data: serde_yaml::Value, template_mode: TemplateMode) -> Result<String, String> {
        match data {
            serde_yaml::Value::Mapping(x) => { return self.render(template, x, template_mode); }
            _ => { panic!("this method requires a mapping"); }
        }
    }

    pub fn test_condition(&self, expr: &String, data: serde_yaml::Mapping, template_mode: TemplateMode) -> Result<bool, String> {
        if (template_mode == TemplateMode::Off) {
            /* this is only used to get back the raw 'items' collection inside the task FSM */
            return Ok(true);
        }
        let template = format!("{{{{#if {expr} }}}}true{{{{ else }}}}false{{{{/if}}}}");
        let result = self.render(&template, data, TemplateMode::Strict);
        match result {
            Ok(x) => { 
                if x.as_str().eq("true") {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            },
            Err(y) => { 
                if y.find("Couldn't read parameter").is_some() {
                    return Err(format!("failed to parse conditional: {}: one or more parameters may be undefined", expr))
                }
                else {
                    return Err(format!("failed to parse conditional: {}: {}", expr, y))
                }
            }
        };
    }

}
