#![feature(raw)]

extern crate png;
extern crate freetype;
extern crate libc;

use std::ptr::null_mut;
use std::ops::Drop;
use std::mem::transmute;
use freetype::freetype::*;

pub struct RenderOption {
    pub size: u32,//font pixel size
}

pub struct CharImageRender {
    pub ft_lib: FT_Library,
    pub ft_face: FT_Face,
}

impl CharImageRender {
    pub fn new() -> CharImageRender {
        let mut ft_lib: FT_Library = null_mut();
        unsafe {
            FT_Init_FreeType(&mut ft_lib);
        }
        let mut ft_face: FT_Face = null_mut();
        let ptr = "./resources/fonts/Hiragino_Sans_GB_W3.ttf".as_ptr();
        unsafe {
            FT_New_Face(ft_lib, ptr as *mut i8, 0, &mut ft_face);
        }
        CharImageRender {
            ft_lib: ft_lib,
            ft_face: ft_face,
        }
    }

    pub fn render(&mut self, ch: char, opt: &RenderOption) -> Box<[u8]> {
        unsafe {
            FT_Set_Pixel_Sizes(self.ft_face, 0, opt.size);
            FT_Load_Char(self.ft_face, ch as u64, FT_LOAD_DEFAULT);

            let slot: FT_GlyphSlot = (*self.ft_face).glyph as FT_GlyphSlot;
            FT_Render_Glyph(slot, FT_RENDER_MODE_NORMAL);
            let bitmap = &(*slot).bitmap;
            self.print_bitmap(&bitmap);
            // println!("{:?}", ((*slot).bitmap_left, (*slot).bitmap_top));
        }
        unimplemented!()
    }

    pub fn print_bitmap(&self, bitmap: &FT_Bitmap) {
        use std::raw::Slice;

        let buffer: &[u8] = unsafe { transmute(
            Slice {
                data: bitmap.buffer,
                len: (bitmap.rows * bitmap.pitch) as usize
            }
        )};
        for i in 0..bitmap.rows {
            for j in 0..bitmap.width {
                let pixel = buffer[(bitmap.pitch * i + j) as usize];
                print!("{:02x}", pixel);
            }
            println!("");
        }
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
