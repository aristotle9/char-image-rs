extern crate png;
extern crate freetype;

mod lib;
use lib::*;

use png::{to_vec, store_png};

fn main() {
    let mut render = CharImageRender::new();
    for c in "abcDEF赵钱孙李周吴郑王".chars() {
        let bitmap = render.render(c, &RenderOption { size: 72, padding: 50});
        // println!("{}", bitmap);
        let mut image = bitmap.to_rgb_png_image(get_bg_color(c));
        // let mut image = bitmap.to_gray_png_image();
        // store_png(&mut image, format!("outputs/img-{:04x}.png", c as u32)).unwrap();
        println!("{} done", c);
    }
}
