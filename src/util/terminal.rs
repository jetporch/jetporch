
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

pub fn three_column_table(headerA: String, headerB: String, headerC: String, elements: Vec<(String,String,String)>) {
    let mut buffer = String::from("|:-|:-|:-\n");
    println!("");
    buffer.push_str(
        &String::from(format!("|{}|{}|{}\n", headerA, headerB, headerC))
    );
    for (a,b,c) in elements.iter() {
        buffer.push_str(&String::from("|-|-|-\n"));
        buffer.push_str(
            &String::from(format!("|{}|{}|{}\n", a, b, c))
        );
    }
    buffer.push_str(&String::from("|-|-|-\n"));
    println!("");
    markdown_print(&buffer);
}
