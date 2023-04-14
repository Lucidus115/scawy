use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

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
            .zip(&tex.pixels[s..s + width]);
        for (left, &right) in zipped {
            if right > 0 {
                *left = right;
            }
        }

        s += width;
    }
}

pub struct Texture {
    width: u32,
    height: u32,
    pixels: Rc<[u8]>,
}

impl Texture {
    pub(crate) fn load(path: &Path) -> Self {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("assets/");
        p.push(path);

        let Ok(im) = image::open(&p) else {
            warn!("Could not find an image with the path: {:?}", path);
            return Self { width: 0, height: 0, pixels: Vec::with_capacity(0).into()};
        };
        Self {
            width: im.width(),
            height: im.height(),
            pixels: im.as_bytes().into(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &Rc<[u8]> {
        &self.pixels
    }
}
