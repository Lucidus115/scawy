use crate::graphics::Texture;
use std::{collections::HashMap, path::Path};

pub struct Assets {
    folder_path: String,
    textures: HashMap<String, Texture>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            folder_path: (env!("CARGO_MANIFEST_DIR").to_owned() + "/assets/"),
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, path: impl Into<String>) {
        let path = path.into();

        let full_path = format!("{}{}", self.folder_path, path);
        let tex = Texture::load(Path::new(full_path.as_str()));

        self.textures.insert(path, tex);
    }

    pub fn get_texture(&self, path: impl Into<String>) -> Option<&Texture> {
        self.textures.get(&path.into())
    }
}
