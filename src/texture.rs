use std::collections::HashMap;

use crate::math;
use image::{self, GenericImageView};

pub struct Texture {
    image: image::DynamicImage,
    id: u32,
}

impl Texture {
    fn load(filename: &str, id: u32) -> image::ImageResult<Texture> {
        Ok(Self {
            image: image::open(filename)?,
            id,
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
}

impl TextureStorage {
    pub fn load(&mut self, filename: &str) -> image::ImageResult<u32> {
        let id = self.cur_id;
        self.cur_id += 1;
        self.images.insert(id, Texture::load(filename, id)?);
        Ok(id)
    }

    pub fn get(&self, id: u32) -> Option<&Texture> {
        self.images.get(&id)
    }
}
