use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path}; // ,PathBuf};
use std::fs::read_to_string;

const YAML_ERROR_SHOW_LINES:usize = 10;

// prints nice explanations of YAML messages.  This is to be called from something else that already
// realizes and handles the error so it doesn't have a meaningful return value. If the YAML error
// has no line/column info it prints a much shorter explanation.

pub fn show_yaml_error_in_context(yaml_error: &serde_yaml::Error, path: &Path) {

    // open the YAML file again so we can print it
    
    // we already tried opening the file once so this should not panic.
    let f = File::open(path).unwrap();

    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    let markdown_table = format!("|:-|\n\
                                  |Error reading YAML file: {}|\n\
                                  |{}|\n\
                                  |-", path.display(), yaml_error);
                        

    crate::util::terminal::markdown_print(&markdown_table);

    /*
    println!("---------------------------------------------------------");
    println!("Error reading YAML: {}", path.display());
    println!("{}", yaml_error);
    println!("---------------------------------------------------------");
    println!("");
    */

    // see if there is a YAML line number in the error structure, if not, we can't show the
    // context in the file
    let location = yaml_error.location();
    if location.is_none() {
        return; 
    }

    // get the line/column info out of the location object
    let location = location.unwrap();
    let mut lines : Vec<String> = Vec::new();
    let error_line = location.line();
    let error_column = location.column();
    let mut line_count: usize = 0;

    let lines: Vec<String> = read_to_string(path).unwrap().lines().map(String::from).collect();

    let line_count = lines.len();


    // figure out what our start and stop line numbers are when showing
    // where the errors are in the YAML
    let mut show_start: usize = 0;
    let mut show_stop: usize = 0;
    
    if error_line < YAML_ERROR_SHOW_LINES {
        show_start = 1;
    }
    show_stop = error_line + YAML_ERROR_SHOW_LINES;
    if show_stop > line_count {
        show_stop = line_count;
    }

    let mut count: usize = 0;

    for line in lines.iter() {
        count = count + 1;
        if count >= show_start && count <= show_stop {
            if count ==  error_line {
                println!("{count:5.0} >>> | {}", line);
            } else {
                println!("{count:5.0}     | {}", line);
            }
        }
    }
    println!("---------------------------------------------------------");
    println!("");

}
