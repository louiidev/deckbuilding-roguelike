use std::collections::HashMap;
use raylib::core::{ RaylibHandle, RaylibThread };
use raylib::core::texture::Texture2D;
use raylib::prelude::*;


pub fn load_texture(
    location: &str,
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
) -> Texture2D {
    let i = Image::load_image(location).expect("could not load image");
    rl.load_texture_from_image(&thread, &i)
        .expect("could not load texture from image")
}

pub struct Textures {
    pub data: HashMap<String, raylib::core::texture::Texture2D>
}


impl Textures {
    pub fn from_paths(to_load: Vec<&str>, mut rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let mut data: HashMap<String, Texture2D> = HashMap::new();
        for l in to_load {
            data.insert(l.to_string(), load_texture(&format!("assets/{}.png", l).to_string(), &mut rl, &thread));
        }
        Textures {
            data
        }
    }
    pub fn get(&self, texture: &str) -> &Texture2D {
        self.data.get(texture).expect("Couldn't load texture")
    }
    pub fn add(&mut self, name: &str, texture: Texture2D) {
        self.data.insert(name.to_string(), texture);
    }
    pub fn contains(&self, name: &str) -> bool {
        self.data.contains_key(name)
    }
}