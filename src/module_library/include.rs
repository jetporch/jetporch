use crate::module_base::common::{IsTask};

define_task!(Include { path: String });
add_task_properties!(Include);

impl IsTask for Include {
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}