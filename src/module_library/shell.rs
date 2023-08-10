use crate::module_base::common::{IsTask};

define_task!(Shell { cmd: String });
add_task_properties!(Shell);

impl IsTask for Shell {
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}