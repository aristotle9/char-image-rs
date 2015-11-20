extern crate png;
extern crate freetype;
extern crate libc;

use std::ptr::null_mut;
use std::ops::Drop;
use freetype::freetype::*;

pub struct RenderOption {
    pub size: i32,//font pixel size
    pub padding: i32,
}

pub struct CharImageRender {
    pub ft_lib: FT_Library,
    pub ft_face: FT_Face,
}

pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub pitch: i32,
    pub buffer: Vec<u8>,
}


impl Bitmap {
    pub fn new(w: i32, h: i32, pitch: i32) -> Self {
        Bitmap {
            w: w,
            h: h,
            pitch: pitch,
            buffer: vec![0; (h * pitch) as usize]
        }
    }

    pub fn from_ft_bitmap(bitmap: &FT_Bitmap) -> Self {
        use std::io::Write;
        use std::slice;

        let w = bitmap.width;
        let h = bitmap.rows;
        let pitch = bitmap.pitch;

        let mut buf: Vec<u8> = Vec::with_capacity((h * pitch) as usize);
        let bitmap_buf = unsafe { slice::from_raw_parts(bitmap.buffer, (h * pitch) as usize) };
        buf.write(bitmap_buf).unwrap();

        assert_eq!(bitmap_buf.len(), (h * pitch) as usize);
        Bitmap {
            w: w,
            h: h,
            pitch: pitch,
            buffer: buf,
        }
    }
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

    pub fn render(&mut self, ch: char, opt: &RenderOption) -> Bitmap {
        let mut img = Bitmap::new(opt.size, opt.size, opt.size);
        unsafe {
            FT_Set_Pixel_Sizes(self.ft_face, 0, opt.size as u32);
            FT_Load_Char(self.ft_face, ch as u64, FT_LOAD_DEFAULT);

            let slot: FT_GlyphSlot = (*self.ft_face).glyph as FT_GlyphSlot;
            FT_Render_Glyph(slot, FT_RENDER_MODE_NORMAL);

            let bitmap = Bitmap::from_ft_bitmap(&(*slot).bitmap);
            Self::bitmap_blit(&bitmap, (opt.size - bitmap.w) / 2, (opt.size - bitmap.h) / 2, &mut img);
        }
        Self::print_bitmap(&img);
        println!("");
        img
    }

    fn bitmap_blit(src: &Bitmap, left: i32, top: i32, dst: &mut Bitmap) {
        use std::cmp::{max, min};

        let x1 = max(left, 0);
        let x2 = min(left + src.w, dst.w);
        if x2 <= x1 {
            return;
        }

        let y1 = max(top, 0);
        let y2 = min(top + src.h, dst.h);
        if y2 <= y1 {
            return;
        }

        let mut y = y1;
        while y < y2 {
            let mut x = x1;
            while x < x2 {
                dst.buffer[(y * dst.pitch + x) as usize] |= src.buffer[((y - top) * src.pitch + (x - left)) as usize];
                x += 1;
            }
            y += 1;
        }
    }

    fn print_bitmap(bitmap: &Bitmap) {
        for i in 0..bitmap.h {
            for j in 0..bitmap.w {
                let pixel = bitmap.buffer[(bitmap.pitch * i + j) as usize];
                if pixel > 0 {
                    print!("{:02x}", pixel);
                } else {
                    print!("..");
                }
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
