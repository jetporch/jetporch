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

use crate::handle::handle::TaskHandle;
use crate::tasks::request::TaskRequest;
use crate::tasks::response::TaskResponse;
use crate::tasks::logic::{PreLogicInput,PreLogicEvaluated,PostLogicEvaluated};
use std::sync::Arc;
use crate::tasks::TemplateMode;

pub struct EvaluatedTask {
    pub action: Arc<dyn IsAction>,
    pub with: Arc<Option<PreLogicEvaluated>>,
    pub and: Arc<Option<PostLogicEvaluated>>
}

pub trait IsTask : Send + Sync { 

    fn get_module(&self) -> String;
    fn get_name(&self) -> Option<String>;
    fn get_with(&self) -> Option<PreLogicInput>;
    
    fn evaluate(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>, tm: TemplateMode) -> Result<EvaluatedTask, Arc<TaskResponse>>;

    fn get_display_name(&self) -> String {
        return match self.get_name() {
            Some(x) => x,
            _ => self.get_module()
        }
    }

}

pub trait IsAction : Send + Sync {

    fn dispatch(&self, handle: &Arc<TaskHandle>, request: &Arc<TaskRequest>) -> Result<Arc<TaskResponse>, Arc<TaskResponse>>;
}

