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
    hb.set_strict_mode(true);
    return hb;
});

pub struct Templar {
}

impl Templar {

    pub fn new() -> Self {
        return Self {
        };
    }

    pub fn render(&self, template: &String, data: serde_yaml::Mapping) -> Result<String, String> {
        //let handlebars = Handlebars::new();
        let result : Result<String, RenderError> = HANDLEBARS.render_template(template, &data);
        return match result {
            Ok(x) => {
                Ok(x)
            },
            Err(y) => {
                Err(format!("Template error: {}", y.desc))
            }
        }
    }

    pub fn test_cond(&self, expr: &String, data: serde_yaml::Mapping) -> Result<bool, String> {
        // see https://docs.rs/handlebars/latest/handlebars/
        let template = format!("{{{{#if {expr} }}}}true{{{{ else }}}}false{{{{/if}}}}");
        let result = self.render(&template, data);
        match result {
            Ok(x) => { 
                if x.as_str().eq("true") {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            },
            Err(_x) => { 
                return Err(format!("failed to parse cond: {}", expr)) 
            }
        };
    }

}
