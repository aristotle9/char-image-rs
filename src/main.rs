extern crate png;
extern crate freetype;
extern crate hyper;
extern crate url;

mod lib;
use lib::*;

use hyper::Server;
use hyper::server::{Request, Response};
use png::{to_vec, store_png};

const RENDER_OPTION: RenderOption = RenderOption { size: 72, padding: 50};

fn png_image_from_char(c: char) -> Vec<u8> {
    let mut render = CharImageRender::new();
    let bmp = render.render(c, &RENDER_OPTION);
    let mut img = bmp.to_rgb_png_image(get_bg_color(c));
    to_vec(&mut img).expect("encoding png err")
}

fn main() {
    Server::http("127.0.0.1:3000").expect("start server failed!").handle(|req: Request, mut res: Response| {
        if req.method == hyper::method::Method::Get {
            let uri = match req.uri {
                hyper::uri::RequestUri::AbsolutePath(uri) => url::percent_encoding::lossy_utf8_percent_decode(uri.as_bytes()),
                _ => String::new()
            };
            let c = uri.chars().nth(1).unwrap_or('X');
            let img = png_image_from_char(c);
            res.headers_mut().set_raw("content-type", vec![b"image/png".to_vec()]);
            res.send(&img).expect("send image failed");
        } else {
            *res.status_mut() = hyper::status::StatusCode::NotFound;
        }
    }).expect("add handle failed");
}
