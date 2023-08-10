

// alphabetized:
use crate::module_library::external::External;
use crate::module_ibrary::include::Include;
use crate::module_library::shell::Shell;

#[derive(Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Task {
    Include(Include),
    Shell(Shell),
    External(External),
}