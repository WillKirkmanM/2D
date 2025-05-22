use std::collections::HashMap;
use std::path::Path;
use image::GenericImageView;

pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct ResourceManager {
    textures: HashMap<String, Texture>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
    
    pub fn load_texture(&mut self, name: &str, path: &str) -> Result<(), String> {
        let img = image::open(Path::new(path))
            .map_err(|e| format!("Failed to load texture: {}", e))?;
            
        let dimensions = img.dimensions();
        let rgba = img.to_rgba8();
        
        let texture = Texture {
            width: dimensions.0,
            height: dimensions.1,
            data: rgba.into_raw(),
        };
        
        self.textures.insert(name.to_string(), texture);
        Ok(())
    }
    
    pub fn get_texture(&self, name: &str) -> Option<&Texture> {
        self.textures.get(name)
    }
}