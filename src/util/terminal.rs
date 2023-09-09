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

#[inline(always)]
pub fn markdown_print(markdown: &String) {
    termimad::print_text(markdown);
}

#[inline(always)]
pub fn banner(msg: &String) {
    let markdown = String::from(format!("|:-|\n\
                                        |{}|\n\
                                        |-", msg));
    markdown_print(&markdown);
}

pub fn two_column_table(header_a: &String, header_b: &String, elements: &Vec<(String,String)>) {
    let mut buffer = String::from("|:-|:-\n");
    buffer.push_str(
        &String::from(format!("|{}|{}\n", header_a, header_b))
    );
    for (a,b) in elements.iter() {
        buffer.push_str(&String::from("|-|-\n"));
        buffer.push_str(
            &String::from(format!("|{}|{}\n", a, b))
        );
    }
    buffer.push_str(&String::from("|-|-\n"));
    markdown_print(&buffer);
}

pub fn captioned_display(caption: &String, body: &String) {
    banner(caption);
    println!("");
    for line in body.lines() {
        println!("    {}", line);
    }
    println!("");
}