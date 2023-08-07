use std::path::{Path}; // ,PathBuf};
use std::fs::read_to_string;

const YAML_ERROR_SHOW_LINES:usize = 10;
const YAML_ERROR_WIDTH:usize = 180; // things will wrap in terminal anyway

// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

pub fn show_yaml_error_in_context(yaml_error: &serde_yaml::Error, path: &Path) {

    // open the YAML file again so we can print it

    // FIXME: may need to trim long error strings as they could contain
    // the whole file (re: yaml_error) inside of format for the error message itself

    // see if there is a YAML line number in the error structure, if not, we can't show the
    // context in the file

    let location = yaml_error.location();

    let mut yaml_error_str = String::from(format!("{}", yaml_error));

    yaml_error_str.truncate(YAML_ERROR_WIDTH);
    if yaml_error_str.len() > YAML_ERROR_WIDTH - 3 {
        yaml_error_str.push_str("...");
    }

    if location.is_none() {
        let markdown_table = format!("|:-|\n\
                                      |Error reading YAML file: {}|\n\
                                      |{}|\n\
                                      |-", path.display(), yaml_error_str);
        crate::util::terminal::markdown_print(&markdown_table);
        return; 
    }

    // get the line/column info out of the location object
    let location = location.unwrap();
    let error_line = location.line();
    let error_column = location.column();

    let lines: Vec<String> = read_to_string(path).unwrap().lines().map(String::from).collect();
    let line_count = lines.len();

    // figure out what our start and stop line numbers are when showing
    // where the errors are in the YAML
    let mut show_start: usize = 0;

    // header showing the error, a blank line, then the file contents exerpt
    let mut markdown_table = String::new();
    markdown_table.push_str(format!("|:-|:-:|:-|\n\
                                 |||Error reading YAML file: {}|\n\
                                 |||{}\n\
                                 |-|-|\n", path.display(), yaml_error_str).as_str());

    if error_line < YAML_ERROR_SHOW_LINES {
        show_start = 1;
    }
    let mut show_stop = error_line + YAML_ERROR_SHOW_LINES;
    if show_stop > line_count {
        show_stop = line_count;
    }

    let mut count: usize = 0;

    for line in lines.iter() {
        count = count + 1;
        if count >= show_start && count <= show_stop {
            if count ==  error_line {
                markdown_table.push_str(format!("|{count}:{error_column} | >>> | {}\n", line).as_str());
            } else {
                markdown_table.push_str(format!("|{count}|| {}\n", line).as_str());
            }
        }
    }

    markdown_table.push_str(format!("|-|-|-\n").as_str());
    crate::util::terminal::markdown_print(&markdown_table);

    println!("");

}
