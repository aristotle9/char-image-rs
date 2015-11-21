extern crate png;
extern crate freetype;

mod lib;

use lib::*;

use png::{to_vec, store_png};

fn main() {
    let mut render = CharImageRender::new();
    for c in "DEFabc你好".chars() {
        let bitmap = render.render(c, &RenderOption { size: 36, padding: 0 });
        println!("{}", bitmap);
        let mut image = bitmap.to_png_image();
        // store_png(&mut image, format!("outputs/img-{}.png", c)).unwrap();
    }
}
