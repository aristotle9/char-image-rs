extern crate png;
extern crate freetype;

mod lib;

use lib::*;

fn main() {
    let mut render = CharImageRender::new();
    render.render('A');
}
