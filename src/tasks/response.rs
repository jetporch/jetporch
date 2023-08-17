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

use std::sync::Arc;
use std::collections::HashMap;

#[derive(PartialEq)]
pub enum TaskStatus {
    IsCreated,
    IsRemoved,
    IsModified,
    IsValidated,
    IsChanged,
    NeedsCreation,
    NeedsRemoval,
    NeedsModification,
    Failed
}

pub struct TaskResponse {
    pub status: TaskStatus,
    pub changes: Arc<Option<HashMap<String, String>>>,
    pub msg: Option<String>,
}

impl TaskResponse {
    pub fn is_failed(&self) -> bool {
        return self.status == TaskStatus::Failed;
    }

    // possible methods for getting change counts, etc?
}