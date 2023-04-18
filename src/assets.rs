use crate::graphics::Texture;
use std::{collections::HashMap, path::Path};

pub struct Assets {
    folder_path: String,
    textures: HashMap<String, Texture>,
}

impl Assets {
    pub fn new() -> Self {
        let path = std::env::current_exe().expect("Failed to find exe path");
        let mut path_str = path.parent().unwrap().to_str().unwrap();
        println!("Path: {:?}", path_str);

        // This is not foolproof
        //TODO: Find a libray to locate resource folder
        if path_str.contains("target") {
            path_str = env!("CARGO_MANIFEST_DIR");
        }
        Self {
            folder_path: format!("{}/assets", path_str),
            textures: HashMap::new(),
        }
    }

    pub fn load_texture(&mut self, path: impl Into<String>) {
        let path = path.into();
        let full_path = format!("{}/{}", self.folder_path, path);
        let tex = Texture::load(Path::new(full_path.as_str()));

        self.textures.insert(path, tex);
    }

    pub fn get_texture(&self, path: impl Into<String>) -> Option<&Texture> {
        self.textures.get(&path.into())
    }
}
