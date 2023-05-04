use assets_manager::{
    loader::{ImageLoader, LoadFrom},
    Asset,
};
use image::DynamicImage;

use crate::WIDTH;

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
            let output = (val as f32 * alpha) + (*self_val as f32 * (1. - alpha));
            *self_val = output as u8;
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

pub fn draw_text(screen: &mut [u8], pos: crate::UVec2, text: &str) {
    let img = image::open("assets/font.png").expect("missing assets/font.png");

    let mut pos = pos;
    for c in text.chars() {
        pos.x += 10;
        blit_sheet(screen, pos, &img, c as usize, 16, 16);
    }
}

fn blit_sheet(
    screen: &mut [u8],
    dest: crate::UVec2,
    sprite: &DynamicImage,
    index: usize,
    rows: usize,
    columns: usize,
) {
    let pixels = sprite.as_bytes();
    let width = sprite.width() as usize / rows * 4;
    let height = sprite.height() as usize / columns;

    // convert index to x and y coords
    let idx_x = index % rows;
    let idx_y = index / rows;

    for y in 0..height {
        // coordinates on the sprite sheet
        let x_pos = idx_x * width;
        let y_pos = idx_y * height * 4;

        let i = dest.x as usize * 4 + dest.y as usize * WIDTH * 4 + y * WIDTH * 4;
        let s = ((y as f32 * 4. + y_pos as f32) * width as f32 * (rows as f32 / 4.) + x_pos as f32)
            as usize;

        // check for transparency
        let pixels = &pixels[s..s + width];
        let colors: Vec<Color> = pixels.chunks(4).map(Color::from).collect();

        let screen_pixels = &screen[i..i + width];
        let mut screen_colors: Vec<Color> = screen_pixels.chunks(4).map(Color::from).collect();

        let mut vec = Vec::with_capacity(screen_pixels.len());
        screen_colors.iter_mut().enumerate().for_each(|(idx, color)| {
            color.blend(colors[idx]);
            let pix = color.slice();

            for i in pix {
                vec.push(i);
            }
        });

        screen[i..i + width].copy_from_slice(&vec);
    }
}

fn blit(screen: &mut [u8], dest: crate::UVec2, sprite: &DynamicImage) {
    blit_sheet(screen, dest, sprite, 0, 1, 1);
}
