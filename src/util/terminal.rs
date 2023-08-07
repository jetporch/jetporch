
// ==============================================================================================================
// PUBLIC API
// ==============================================================================================================

pub fn markdown_print(markdown: &String) {
    termimad::print_text(markdown);
}

pub fn banner(msg: String) {
    let markdown = String::from(format!("|:-|\n\
                                        |{}|\n\
                                        |-", msg));
    markdown_print(&markdown);
}

pub fn two_column_table(label_header: String, value_header: String, elements: Vec<(String,String)>) {
    let mut buffer = String::from("|:-|:-|\n");
    println!("");
    buffer.push_str(
        &String::from(format!("|{}|{}|\n", label_header, value_header))
    );
    for (a,b) in elements.iter() {
        buffer.push_str(&String::from("|-|-|\n"));
        buffer.push_str(
            &String::from(format!("|{}|{}|\n", a, b))
        );
    }
    buffer.push_str(&String::from("|-|-|\n"));
    println!("");
    markdown_print(&buffer);
}
