use crate::module_base::common::{IsTask};

define_task!(External { module: String, params: HashMap<String, Value> });
add_task_properties!(External);

impl IsTask for External {

    // FIXME: this is just an example function signature
    fn run(&self) -> Result<(), String> {
        return Ok(());
    }
}