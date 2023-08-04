
//use termimad::crossterm::{execute, style::Color::*, terminal};
//use termimad::*;

pub fn markdown_print(markdown: &String) {

    //let mut skin = MadSkin::default();
    //skin.set_headers_fg(rgb(255, 187, 0));
    //skin.bold.set_fg(Yellow);
    //skin.italic.set_fgbg(Magenta, rgb(30, 30, 40));
    //skin.bullet = StyledChar::from_fg_char(Yellow, '⟡');
    //skin.quote_mark.set_fg(Yellow);

    termimad::print_text(markdown);


}