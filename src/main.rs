extern crate png;
extern crate freetype;

mod lib;

use lib::*;

fn main() {
    let mut render = CharImageRender::new();
    for c in "ABCacb你好".chars() {
        render.render(c, &RenderOption { size: 36, padding: 0 });
    }
}
