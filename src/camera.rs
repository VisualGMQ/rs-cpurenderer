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

    pub fn near(&self) -> f32 {
        self.near
    }
}

pub struct Camera {
    frustum: Frustum,
    position: math::Vec3,
    rotation: math::Vec3,

    view_mat: math::Mat4,
    view_dir: math::Vec3,
}

impl Camera {
    pub fn new(near: f32, far: f32, aspect: f32, fovy: f32) -> Self {
        Self {
            frustum: Frustum::new(near, far, aspect, fovy),
            position: math::Vec3::new(0.0, 0.0, 0.0),
            view_mat: math::Mat4::identity(),
            rotation: math::Vec3::zero(),
            view_dir: -*math::Vec3::z_axis(),
        }
    }

    pub fn get_frustum(&self) -> &Frustum {
        &self.frustum
    }

    pub fn move_to(&mut self, position: math::Vec3) {
        self.position = position;
        self.recalc_view_mat();
    }

    pub fn move_offset(&mut self, offset: math::Vec3) {
        self.position += offset;
        self.recalc_view_mat();
    }

    pub fn position(&self) -> &math::Vec3 {
        &self.position
    }

    #[rustfmt::skip]
    pub fn lookat(&mut self, target: math::Vec3) {
        let back = (self.position - target).normalize();
        let up = math::Vec3::y_axis();
        let right = up.cross(&back).normalize();
        let up = back.cross(&right).normalize();

        self.view_mat = math::Mat4::from_row(&[
            right.x, right.y, right.z, -right.dot(&self.position),
               up.x,    up.y,    up.z,    -up.dot(&self.position),
             back.x,  back.y,  back.z,  -back.dot(&self.position),
                0.0,     0.0,     0.0,                        1.0,
        ]);

        let dir = target - self.position;
        let x = math::Vec3::y_axis().dot(&math::Vec3::new(0.0, dir.y, dir.z).normalize()).acos();
        let y = math::Vec3::z_axis().dot(&math::Vec3::new(dir.x, 0.0, dir.z).normalize()).acos();
        let z = math::Vec3::x_axis().dot(&math::Vec3::new(dir.x, dir.y, 0.0).normalize()).acos();
        self.view_dir = -back;
        self.rotation = math::Vec3::new(x, y, z);
    }

    pub fn set_rotation(&mut self, rotation: math::Vec3) {
        self.rotation = rotation;
        self.recalc_view_mat();
    }

    fn recalc_view_mat(&mut self) {
        let rotation = math::create_eular_rotate_xyz(&-self.rotation);
        self.view_mat = rotation * math::create_translate(&-self.position);
        self.view_dir = (rotation * math::Vec4::new(0.0, 0.0, -1.0, 1.0)).truncated_to_vec3();
    }

    pub fn get_rotation(&self) -> &math::Vec3 {
        &self.rotation
    }

    pub fn view_mat(&self) -> &math::Mat4 {
        &self.view_mat
    }

    pub fn view_dir(&self) -> &math::Vec3 {
        &self.view_dir
    }
}
