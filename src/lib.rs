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

    #[inline]
    fn get_image_size(&self) -> i32 {
        self.size + self.padding
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
        assert!(ft_face != null_mut());
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

    pub fn render_svg(&mut self, ch: char, opt: &RenderOption, bg_color: u32) -> String {
        unsafe {
            FT_Set_Pixel_Sizes(self.ft_face, 0, opt.size as u32);
            FT_Load_Char(self.ft_face, ch as u64, FT_LOAD_DEFAULT);

            let slot: FT_GlyphSlot = (*self.ft_face).glyph as FT_GlyphSlot;
            let mut bbox: FT_BBox = struct_FT_BBox_ {
                xMin: 0,
                yMin: 0,
                xMax: 0,
                yMax: 0,
            };
            FT_Outline_Get_BBox(&mut (*slot).outline, &mut bbox);
            let path = Self::svg_path(&mut (*slot).outline);

            Self::svg_img(path, &bbox, opt, bg_color)
        }
    }

    fn svg_path(outline: &mut FT_Outline) -> String {
        use std::io::Write;

        let funcs: FT_Outline_Funcs = struct_FT_Outline_Funcs_ {
            move_to: ft_outline_move_to_func as *mut u8,
            line_to: ft_outline_line_to_func as *mut u8,
            conic_to: ft_outline_conic_to_funct as *mut u8,
            cubic_to: ft_outline_cubic_to_funct as *mut u8,
            shift: 0,
            delta: 0,
        };
        let mut path_elems: Box<Vec<SVGPathElem>> = Box::new(Vec::new());
        unsafe {
            FT_Outline_Decompose(outline, &funcs, &mut *path_elems as *mut _ as *mut libc::c_void);
        }
        let mut buf = Vec::new();
        for e in path_elems.iter() {
            write!(&mut buf, "{}", e);
        }
        String::from_utf8(buf).unwrap()
    }

    fn svg_img(path: String, bbox: &FT_BBox, opt: &RenderOption, bg_color: u32) -> String {
        let x1 = bbox.xMin as f64 / 64f64;
        let x2 = bbox.xMax as f64 / 64f64;
        let y1 = bbox.yMin as f64 / 64f64;
        let y2 = bbox.yMax as f64 / 64f64;
        let w = x2 - x1;
        let h = y2 - y1;
        let dx = (opt.get_image_size() as f64 - w) / 2f64;
        let dy = (opt.get_image_size() as f64 - h) / 2f64;
        format!(r#"<?xml version='1.0' encoding='UTF-8'?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.0//EN" "http://www.w3.org/TR/2001/REC-SVG-20010904/DTD/svg10.dtd">
<svg xmlns='http://www.w3.org/2000/svg' version='1.0' width='{}' height='{}'>
    <rect width="100%" height="100%" style="fill: #{:06x}" />
    <g transform='scale(1 -1) translate({} {})'>
        <path d='{}' style='fill: #ffffff'/>
    </g>
</svg>
"#, opt.get_image_size(), opt.get_image_size(), bg_color, -x1 + dx, -y2 - dy, path)
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

#[derive(Debug)]
enum SVGPathElem {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    ConicTo(f64, f64, f64, f64),
    CubicTo(f64, f64, f64, f64, f64, f64),
}

impl fmt::Display for SVGPathElem {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            SVGPathElem::MoveTo(x, y) => write!(f, "M{} {}\n", x, y),
            SVGPathElem::LineTo(x, y) => write!(f, "L{} {}\n", x, y),
            SVGPathElem::ConicTo(cx, cy, x, y) => write!(f, "Q{} {} {} {}\n", cx, cy, x, y),
            SVGPathElem::CubicTo(c1x, c1y, c2x, c2y, x, y) => write!(f, "C{} {} {} {} {} {}\n", c1x, c1y, c2x, c2y, x, y),
        }
    }
}

extern fn ft_outline_move_to_func(to: *const FT_Vector, user: *mut libc::c_void) -> libc::c_int {
    let elems: &mut Vec<SVGPathElem> = unsafe { &mut *(user as *mut Vec<SVGPathElem>) };
    let to: &FT_Vector = unsafe { &*to };
    elems.push(SVGPathElem::MoveTo(to.x as f64 / 64f64, to.y as f64 / 64f64));
    0
}

extern fn ft_outline_line_to_func(to: *const FT_Vector, user: *mut libc::c_void) -> libc::c_int {
    let elems: &mut Vec<SVGPathElem> = unsafe { &mut *(user as *mut Vec<SVGPathElem>) };
    let to: &FT_Vector = unsafe { &*to };
    elems.push(SVGPathElem::LineTo(to.x as f64 / 64f64, to.y as f64 / 64f64));
    0
}

extern fn ft_outline_conic_to_funct(control: *const FT_Vector, to: *const FT_Vector, user: *mut libc::c_void) -> libc::c_int {
    let elems: &mut Vec<SVGPathElem> = unsafe { &mut *(user as *mut Vec<SVGPathElem>) };
    let control: &FT_Vector = unsafe { &*control };
    let to: &FT_Vector = unsafe { &*to };
    elems.push(SVGPathElem::ConicTo(control.x as f64 / 64f64, control.y as f64 / 64f64, to.x as f64 / 64f64, to.y as f64 / 64f64));
    0
}

extern fn ft_outline_cubic_to_funct(control1: *const FT_Vector, control2: *const FT_Vector, to: *const FT_Vector, user: *mut libc::c_void) -> libc::c_int {
    let elems: &mut Vec<SVGPathElem> = unsafe { &mut *(user as *mut Vec<SVGPathElem>) };
    let control1: &FT_Vector = unsafe { &*control1 };
    let control2: &FT_Vector = unsafe { &*control2 };
    let to: &FT_Vector = unsafe { &*to };
    elems.push(SVGPathElem::CubicTo(control1.x as f64 / 64f64, control1.y as f64 / 64f64, control2.x as f64 / 64f64, control2.y as f64 / 64f64, to.x as f64 / 64f64, to.y as f64 / 64f64));
    0
}

#[link(name="freetype")]
extern {
    pub fn FT_Outline_Decompose(outline: *mut FT_Outline, func_interface: *const FT_Outline_Funcs, user: *mut libc::c_void) -> FT_Error;
    pub fn FT_Outline_Get_BBox(outline: *mut FT_Outline, abbox: *mut FT_BBox) -> FT_Error;
}
