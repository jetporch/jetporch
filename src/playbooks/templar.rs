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

use handlebars::Handlebars;
use handlebars::RenderError;
use std::collections::HashMap;
use serde_yaml;

pub struct Templar {
}

impl Templar {

    pub fn new() -> Self {
        Self {
        }
    }

    pub fn render(&self, template: &String, data: HashMap<String,serde_yaml::Value>) -> Result<String, String> {
        //let value_result = HashMap<String,serde_yaml::Value> = serde_yaml::from_str(&yaml); 
        //if value_result.is_err() {
        //    return Err("internal error: YAML deserialization of internal state failed");
        //}
        let handlebars = Handlebars::new();
        let result : Result<String, RenderError> = handlebars.render_template(template, &data);
        return match result {
            Ok(x) => Ok(x),
            Err(y) => Err(format!("Template error: line {:?}: {}", y.line_no, y.desc))
        }
    }

}
