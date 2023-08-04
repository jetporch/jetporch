use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path}; // ,PathBuf};

const YAML_ERROR_SHOW_LINES:u32 = 10;

// prints nice explanations of YAML messages.  This is to be called from something else that already
// realizes and handles the error so it doesn't have a meaningful return value. If the YAML error
// has no line/column info it prints a much shorter explanation.

pub fn show_yaml_error_in_context(yaml_error: &serde_yaml::Error, path: &Path) {

    // open the YAML file again so we can print it
    
    // we already tried opening the file once so this should not panic.
    let f = File::open(path).unwrap();

    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    println!("");
    println!("---------------------------------------------------------");
    println!("Error reading YAML: {}", path.display());
    println!("{}", yaml_error);
    println!("---------------------------------------------------------");
    println!("");

    // see if there is a YAML line number in the error structure, if not, we can't show the
    // context in the file
    let location = yaml_error.location();
    if location.is_none() {
        return; 
    }

    println!("---------------------------------------------------------");
    println!("Error reading YAML: {}", path.display());
    println!("{}", yaml_error);
    println!("---------------------------------------------------------");

    // get the line/column info out of the location object
    let location = location.unwrap();
    let lines : Vec<String> = Vec::new();
    let error_line = location.line();
    let error_column = location.column();
    let mut line_count: u32 = 0;

    // store all the lines in the buffer
    // FIMXE: refactor, this is just quick to get things done
    while let std::result::Result::Ok(line) = reader.read_line(&mut buffer) {
        line_count = line_count + 1;
        lines.push(line?.trim().clone())
    }

    // figure out what our start and stop line numbers are when showing
    // where the errors are in the YAML
    let show_start: u32 = 0;
    let show_stop: u32 = 0;
    
    if error_line < YAML_ERROR_SHOW_LINES {
        show_start = 1;
    }
    show_stop = error_line + YAML_ERROR_SHOW_LINES;
    if show_stop > line_count {
        show_stop = line_count;
    }

    let mut count: u32 = 0;

    for line in lines.iter() {
        count = count + 1;
        if count >= show_start && count <= show_stop {
            if count == line {
                println!("{count:5.0} >>> | {}", line);
            } else {
                println!("{count:5.0}     | {}", line);
            }
        }
    }
    println!("---------------------------------------------------------");
    println!("");

}
