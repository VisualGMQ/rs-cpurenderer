use std::collections::HashMap;

use crate::math;
use image::{self, GenericImageView};

pub struct Texture {
    image: image::DynamicImage,
    id: u32,
    name: String,
}

impl Texture {
    fn load(filename: &str, id: u32, name: &str) -> image::ImageResult<Texture> {
        Ok(Self {
            image: image::open(filename)?,
            id,
            name: name.to_string(),
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, x: u32, y: u32) -> math::Vec4 {
        let pixel = self.image.get_pixel(x, y);
        let data = &pixel.0;
        math::Vec4::new(
            data[0] as f32 / 255.0,
            data[1] as f32 / 255.0,
            data[2] as f32 / 255.0,
            data[3] as f32 / 255.0,
        )
    }
}

#[derive(Default)]
pub struct TextureStorage {
    cur_id: u32,
    images: HashMap<u32, Texture>,
    name_id_map: HashMap<String, u32>,
}

impl TextureStorage {
    pub fn load(&mut self, filename: &str, name: &str) -> image::ImageResult<u32> {
        let id = self.cur_id;
        self.cur_id += 1;
        self.images.insert(id, Texture::load(filename, id, name)?);
        self.name_id_map.insert(name.to_string(), id);
        Ok(id)
    }

    pub fn get_by_id(&self, id: u32) -> Option<&Texture> {
        self.images.get(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Texture> {
        let id = self.name_id_map.get(name)?;
        self.images.get(id)
    }

    pub fn get_id(&self, name: &str) -> Option<&u32> {
        self.name_id_map.get(name)
    }
}
