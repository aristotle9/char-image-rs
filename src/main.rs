extern crate png;
extern crate freetype;
extern crate iron;
extern crate url;

mod lib;
use lib::*;

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::borrow::Borrow;

use iron::prelude::Request;
use iron::prelude::Response;
use iron::prelude::Iron;
use iron::prelude::IronResult;
use iron::middleware::Handler;
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

struct CharImageHandler {
    cache: RwLock<BTreeMap<char, Vec<u8>>>,
}

impl CharImageHandler {
    fn new() -> Self {
        CharImageHandler {
            cache: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Handler for CharImageHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        if req.method == iron::method::Method::Get {
            let path = req.url.path.first().unwrap();
            let path = url::percent_encoding::lossy_utf8_percent_decode(path.as_bytes());
            let c = path.chars().nth(0).unwrap_or('X');

            let has_cache = self.cache.read().unwrap().contains_key(&c);
            if !has_cache {
                let img = png_image_from_char(c);
                self.cache.write().unwrap().insert(c, img);
            }

            let cache = self.cache.read().unwrap();
            let img = cache.get(&c).unwrap();
            let content_type = "image/png".parse::<Mime>().unwrap();
            Ok(Response::with((content_type, status::Ok, Borrow::<[u8]>::borrow(img))))
        } else {
            Ok(Response::with((status::NotFound)))
        }
    }
}

fn main() {
    let handle = CharImageHandler::new();
    Iron::new(handle).http("127.0.0.1:3000").expect("start server failed");
}
