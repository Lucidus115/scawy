use std::{borrow::Cow, net::IpAddr, path::Path};

use assets_manager::{
    loader::{self, ImageLoader, LoadFrom, Loader, ParseLoader},
    Asset, BoxedError,
};
use image::DynamicImage;
use log::*;

//pub struct Graphics;

pub fn draw_sprite(screen: &mut [u8], pos: &crate::Vec2, tex: &Texture) {
    let width = tex.width() as usize * 4;

    let mut s = 0;
    for y in 0..tex.height() as usize {
        let i = pos.x.floor() as usize * 4
            + pos.y.floor() as usize * crate::WIDTH * 4
            + y * crate::WIDTH * 4;

        // Merge pixels from sprite into screen
        let zipped = screen[i..i + width]
            .iter_mut()
            .zip(&tex.pixels()[s..s + width]);
        for (left, &right) in zipped {
            if right > 0 {
                *left = right;
            }
        }

        s += width;
    }
}

pub struct Texture {
    image: DynamicImage,
}

impl From<DynamicImage> for Texture {
    fn from(value: DynamicImage) -> Self {
        Texture { image: value }
    }
}

impl Asset for Texture {
    const EXTENSIONS: &'static [&'static str] = &["png", "jpg"];
    type Loader = LoadFrom<DynamicImage, ImageLoader>;
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn pixels(&self) -> &[u8] {
        self.image.as_bytes()
    }
}
