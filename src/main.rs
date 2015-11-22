extern crate png;
extern crate freetype;
extern crate iron;
extern crate url;

mod lib;
use lib::*;

use iron::prelude::Request;
use iron::prelude::Response;
use iron::prelude::Iron;
use iron::mime::Mime;
use iron::status;
use png::{to_vec, store_png};

const RENDER_OPTION: RenderOption = RenderOption { size: 72, padding: 50};

fn png_image_from_char(c: char) -> Vec<u8> {
    let mut render = CharImageRender::new();
    let bmp = render.render(c, &RENDER_OPTION);
    let mut img = bmp.to_rgb_png_image(get_bg_color(c));
    to_vec(&mut img).expect("encoding png err")
}

fn main() {
    Iron::new(|req: &mut Request| {
        if req.method == iron::method::Method::Get {
            let path = req.url.path.first().unwrap();
            let path = url::percent_encoding::lossy_utf8_percent_decode(path.as_bytes());
            let c = path.chars().nth(0).unwrap_or('X');
            let img = png_image_from_char(c);
            let content_type = "image/png".parse::<Mime>().unwrap();
            Ok(Response::with((content_type, status::Ok, img)))
        } else {
            Ok(Response::with((status::NotFound)))
        }
    }).http("127.0.0.1:3000").expect("start server failed");
}
