extern crate png;
extern crate freetype;
extern crate libc;

use std::ops::Drop;
use std::mem::size_of;
use freetype::freetype::*;

pub struct CharImageRender {
    pub ft_lib: FT_Library,
    pub ft_face: FT_Face,
}

impl CharImageRender {
    pub fn new() -> CharImageRender {
        let mut ft_lib: FT_Library = unsafe {
            libc::malloc(size_of::<FT_Library>())
        };
        unsafe {
            FT_Init_FreeType(&mut ft_lib);
        }
        let mut ft_face: FT_Face = unsafe {
            libc::malloc(size_of::<FT_Face>()) as FT_Face
        };
        let ptr = "./resources/fonts/Hiragino_Sans_GB_W3.ttf".as_ptr();
        unsafe {
            FT_New_Face(ft_lib, ptr as *mut i8, 0, &mut ft_face);
        }
        CharImageRender {
            ft_lib: ft_lib,
            ft_face: ft_face,
        }
    }

    pub fn render(&mut self, ch: char) -> Box<[u8]> {
        unimplemented!()
    }
}

impl Drop for CharImageRender {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.ft_face);
            FT_Done_FreeType(self.ft_lib);
        }
    }
}
