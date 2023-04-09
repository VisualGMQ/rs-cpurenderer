use crate::math;

pub struct Frustum {
    near: f32,
    aspect: f32,
    fovy: f32,

    mat: math::Mat4,
}

impl Frustum {
    #[rustfmt::skip]
    pub fn new(near: f32, aspect: f32, fovy: f32) -> Self {
        let a = 1.0 / (near * fovy.tan());
        Self {
            near,
            aspect,
            fovy,
            mat: math::Mat4::from_row(&[
                  a,        0.0,         0.0, 0.0,
                0.0, aspect * a,         0.0, 0.0,
                0.0,        0.0,         1.0, 0.0,
                0.0,        0.0, -1.0 / near, 0.0,
            ]),
        }
    }

    pub fn get_mat(&self) -> &math::Mat4 {
        &self.mat
    }
}

pub struct Camera {
    frustum: Frustum,
}

impl Camera {
    pub fn new(near: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            frustum: Frustum::new(near, aspect, fovy),
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }
}
