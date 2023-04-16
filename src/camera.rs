use crate::math;

pub struct Frustum {
    near: f32,
    far: f32,
    aspect: f32,
    fovy: f32,

    mat: math::Mat4,
}

impl Frustum {
    #[rustfmt::skip]
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            near,
            far,
            aspect,
            fovy,
            mat: if cfg!(feature="cpu") {
                let a = 1.0 / (near * fovy.tan());
                // without far plane, clamp x,y in [-1, 1], z = near
                math::Mat4::from_row(&[
                    a,          0.0,         0.0, 0.0,
                    0.0, aspect * a,         0.0, 0.0,
                    0.0,        0.0,         1.0, 0.0,
                    0.0,        0.0, -1.0 / near, 0.0,
                ])
            } else {
                let half_w = near * fovy.tan();
                let half_h = half_w / aspect;
                // with far plane, clamp x,y,z in [-1, 1]
                math::Mat4::from_row(&[
                    near / half_w,           0.0,                       0.0,                             0.0,
                              0.0, near / half_h,                       0.0,                             0.0,
                              0.0,           0.0, far + near / (far - near), 2.0 * far * near / (far - near),
                              0.0,           0.0,                      -1.0,                             0.0,
                ])
            },
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
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            frustum: Frustum::new(near, far, aspect, fovy),
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }
}
