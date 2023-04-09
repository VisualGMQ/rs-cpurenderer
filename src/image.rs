use crate::math;

pub struct PureElemImage<T> {
    data: Vec<T>,
    w: u32,
    h: u32,
}

impl<T> PureElemImage<T> {
    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn in_box(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.w.try_into().unwrap() && y >= 0 && y < self.h.try_into().unwrap()
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }
}

impl PureElemImage<u8> {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            data: vec![0; (w * 3 * h) as usize],
            w,
            h,
        }
    }

    pub fn clear(&mut self, color: &math::Vec4) {
        for x in 0..self.w {
            for y in 0..self.h {
                self.set(x, y, color);
            }
        }
    }

    pub fn set(&mut self, x: u32, y: u32, color: &math::Vec4) {
        self.data[(x + y * self.w) as usize * 3] = (color.x * 255.0) as u8;
        self.data[(x + y * self.w) as usize * 3 + 1] = (color.y * 255.0) as u8;
        self.data[(x + y * self.w) as usize * 3 + 2] = (color.z * 255.0) as u8;
    }
}

impl PureElemImage<f32> {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            data: vec![std::f32::MAX; (w * h) as usize],
            w,
            h,
        }
    }

    pub fn clear(&mut self, value: f32) {
        self.data.fill(value);
    }

    pub fn set(&mut self, x: u32, y: u32, value: f32) {
        self.data[(x + y * self.w) as usize] = value;
    }

    pub fn get(&self, x: u32, y: u32) -> f32 {
        self.data[(x + y * self.w) as usize]
    }
}

pub type ColorAttachment = PureElemImage<u8>;
pub type DepthAttachment = PureElemImage<f32>;
