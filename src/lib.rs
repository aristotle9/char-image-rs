extern crate png;
extern crate freetype;
extern crate libc;

use std::ptr::null_mut;
use std::ops::Drop;
use std::fmt;

use freetype::freetype::*;
use png::{Image, PixelsByColorType};

pub struct RenderOption {
    pub size: i32,//font pixel size
    pub padding: i32,
}

impl RenderOption {
    fn get_blank_bitmap(&self) -> Bitmap {
        let a = self.size + self.padding;
        Bitmap::new(a, a, a)
    }
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

pub fn get_bg_color(c: char) -> u32 {
    match (c as u32) % 9 {
        0 => 0x45BDF3,
        1 => 0xE08F70,
        2 => 0x4DB6AC,
        3 => 0x9575CD,
        4 => 0xB0855E,
        5 => 0xF06292,
        6 => 0xA3D36C,
        7 => 0x7986CB,
        _ => 0xF1B91D,
    }
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

    pub fn to_gray_png_image(self) -> Image {
        Image {
            width: self.w as u32,
            height: self.h as u32,
            pixels: PixelsByColorType::K8(self.buffer),
        }
    }

    pub fn to_rgb_png_image(self, bg_color: u32) -> Image {
        let r = (bg_color >> 16) & 0xff;
        let g = (bg_color >>  8) & 0xff;
        let b = bg_color & 0xff;
        let pixels: Vec<u8> = self.buffer.into_iter().flat_map(|c| {
            if c == 0 {
                vec![r as u8, g as u8, b as u8].into_iter()
            } else {
                let fac = (255f64 - c as f64) / 255f64;
                vec![(c as f64 + (r as f64 * fac)) as u8,
                     (c as f64 + (g as f64 * fac)) as u8,
                     (c as f64 + (b as f64 * fac)) as u8].into_iter()
            }
        }).collect();
        Image {
            width: self.w as u32,
            height: self.h as u32,
            pixels: PixelsByColorType::RGB8(pixels)
        }
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
}

impl fmt::Display for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for i in 0..self.h {
            for j in 0..self.w {
                let pixel = self.buffer[(self.pitch * i + j) as usize];
                if pixel > 0 {
                    try!(write!(f, "{:02x}", pixel));
                } else {
                    try!(write!(f, ".."));
                }
            }
            try!(write!(f, "\r\n"));
        }
        Ok(())
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
        let mut img = opt.get_blank_bitmap();
        unsafe {
            FT_Set_Pixel_Sizes(self.ft_face, 0, opt.size as u32);
            FT_Load_Char(self.ft_face, ch as u64, FT_LOAD_DEFAULT);

            let slot: FT_GlyphSlot = (*self.ft_face).glyph as FT_GlyphSlot;
            FT_Render_Glyph(slot, FT_RENDER_MODE_NORMAL);

            let bitmap = Bitmap::from_ft_bitmap(&(*slot).bitmap);
            Bitmap::bitmap_blit(&bitmap, (img.w - bitmap.w) / 2, (img.h - bitmap.h) / 2, &mut img);
        }
        img
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
