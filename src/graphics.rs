use assets_manager::{
    loader::{ImageLoader, LoadFrom},
    Asset,
};
use image::DynamicImage;

use crate::{WIDTH, idx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn blend(&mut self, color: Color) {
        let inner_blend = |self_val: &mut u8, val: u8| {
            let alpha = color.a as f32 / 255.;
            *self_val = (val as f32 - (1. - alpha) * *self_val as f32 / alpha) as u8;
        };
        inner_blend(&mut self.r, color.r);
        inner_blend(&mut self.g, color.g);
        inner_blend(&mut self.b, color.b);
    }

    pub fn slice(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl From<&[u8]> for Color {
    fn from(value: &[u8]) -> Self {
        Self {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

pub struct Texture {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl From<DynamicImage> for Texture {
    fn from(value: DynamicImage) -> Self {
        Texture {
            width: value.width(),
            height: value.height(),
            bytes: value.as_bytes().into(),
        }
    }
}

impl Asset for Texture {
    const EXTENSIONS: &'static [&'static str] = &["png", "jpg"];
    type Loader = LoadFrom<DynamicImage, ImageLoader>;
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixel(&self, idx: usize) -> Color {
        Color {
            r: self.bytes[idx],
            g: self.bytes[idx + 1],
            b: self.bytes[idx + 2],
            a: self.bytes[idx + 3],
        }
    }
}

// I never got this to work lol
pub fn draw_text(screen: &mut [u8], pos: crate::UVec2, text: &str) {

    // Width of each letter multipied by 4 (because rgba)
    const L_WIDTH: usize = 10 * 4;
    const L_HEIGHT: usize = 12;

    let img = image::open("assets/font.png").expect("missing assets/font.png");
    let buf = img.as_bytes();

    let mut pos_x = pos.x as usize;
    let pos_y = pos.y as usize;
    
    text.chars().for_each(|c| {
        
        let mut s = c as usize;

        for y in 0..img.height() {
            let i = pos_x * 4 + pos_y * crate::WIDTH * 4;
        
            let idx = idx(s as u32 * 4, y * 4, img.width());
    
            // Merge pixels into screen
            let zipped = screen[i..i + 4].iter_mut().zip(&buf[idx..idx + 4]);
            for (left, &right) in zipped {
                if right > 0 {
                    *left = right;
                }
            }
            s += img.width() as usize;
        }

        pos_x += L_WIDTH;
    });
}
