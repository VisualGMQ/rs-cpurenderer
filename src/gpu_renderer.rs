use crate::{camera, image::ColorAttachment, math, renderer::*};

pub struct Renderer {
    color_attachment: ColorAttachment,
    camera: camera::Camera,
    viewport: Viewport,
}

impl RendererInterface for Renderer {
    fn clear(&mut self, color: &math::Vec4) {
        self.color_attachment.clear(color);
    }

    fn get_canva_width(&self) -> u32 {
        self.color_attachment.width()
    }

    fn get_canva_height(&self) -> u32 {
        self.color_attachment.height()
    }

    fn get_rendered_image(&self) -> &[u8] {
        self.color_attachment.data()
    }

    fn draw_triangle(
        &mut self,
        model: &math::Mat4,
        vertices: &[math::Vec3; 3],
        color: &math::Vec4,
    ) {
        // 1. convert 3D coordination to Homogeneous coordinates
        let mut vertices = vertices.map(|v| math::Vec4::from_vec3(&v, 1.0));

        // 2. MVP transform
        for v in &mut vertices {
            *v = *self.camera.get_frustum().get_mat() * *model * *v;
            *v /= v.w;
        }

        // 3. Viewport transform
        let vertices = vertices.map(|v| {
            math::Vec2::new(
                (v.x + 1.0) * 0.5 * (self.viewport.w as f32 - 1.0) + self.viewport.x as f32,
                self.viewport.h as f32 - (v.y + 1.0) * 0.5 * (self.viewport.h as f32 - 1.0)
                    + self.viewport.y as f32,
            )
        });

        // 4. find AABB for triangle
        let aabb_min_x = vertices
            .iter()
            .fold(std::f32::MAX, |min, v| if v.x < min { v.x } else { min })
            .ceil()
            .max(0.0);
        let aabb_min_y = vertices
            .iter()
            .fold(std::f32::MAX, |min, v| if v.y < min { v.y } else { min })
            .ceil()
            .max(0.0);
        let aabb_max_x = vertices
            .iter()
            .fold(std::f32::MIN, |max, v| if v.x > max { v.x } else { max })
            .floor()
            .min(self.color_attachment.width() as f32 - 1.0);
        let aabb_max_y = vertices
            .iter()
            .fold(std::f32::MIN, |max, v| if v.y > max { v.y } else { max })
            .floor()
            .min(self.color_attachment.height() as f32 - 1.0);
        let aabb_min = math::Vec2::new(aabb_min_x, aabb_min_y);
        let aabb_max = math::Vec2::new(aabb_max_x, aabb_max_y);

        // 5. walk through all pixel in AABB and set color
        for x in aabb_min.x as u32..=aabb_max.x as u32 {
            for y in aabb_min.y as u32..=aabb_max.y as u32 {
                if is_pt_in_triangle(&math::Vec2::new(x as f32, y as f32), &vertices) {
                    self.color_attachment.set(x, y, color);
                }
            }
        }
    }
}

impl Renderer {
    pub fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            camera,
            viewport: Viewport { x: 0, y: 0, w, h },
        }
    }
}

fn is_pt_in_triangle(pt: &math::Vec2, vertices: &[math::Vec2; 3]) -> bool {
    let s1 = (vertices[1] - vertices[0]).cross(&(*pt - vertices[0]));
    let s2 = (vertices[2] - vertices[1]).cross(&(*pt - vertices[1]));
    let s3 = (vertices[0] - vertices[2]).cross(&(*pt - vertices[2]));

    (s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0) ||
    (s1 < 0.0 && s2 < 0.0 && s3 < 0.0)
}
