use crate::{camera, image::ColorAttachment, math, renderer::*, vertex::Vertex};

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
        vertices: &[Vertex],
        count: u32,
        texture: Option<&image::DynamicImage>,
    ) {
        for i in 0..count as usize {
            // 1. convert 3D coordination to Homogeneous coordinates
            let mut vertices = [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]];

            // 2. MVP transform
            for v in &mut vertices {
                v.position = *self.camera.get_frustum().get_mat() * *model * v.position;
                v.position /= v.position.w;
            }

            // 3. Viewport transform
            for v in &mut vertices {
                v.position.x = (v.position.x + 1.0) * 0.5 * (self.viewport.w as f32 - 1.0)
                    + self.viewport.x as f32;
                v.position.y = self.viewport.h as f32
                    - (v.position.y + 1.0) * 0.5 * (self.viewport.h as f32 - 1.0)
                    + self.viewport.y as f32;
            }

            // 4. find AABB for triangle
            let aabb_min_x = vertices
                .iter()
                .fold(std::f32::MAX, |min, v| {
                    if v.position.x < min {
                        v.position.x
                    } else {
                        min
                    }
                })
                .ceil()
                .max(0.0);
            let aabb_min_y = vertices
                .iter()
                .fold(std::f32::MAX, |min, v| {
                    if v.position.y < min {
                        v.position.y
                    } else {
                        min
                    }
                })
                .ceil()
                .max(0.0);
            let aabb_max_x = vertices
                .iter()
                .fold(std::f32::MIN, |max, v| {
                    if v.position.x > max {
                        v.position.x
                    } else {
                        max
                    }
                })
                .floor()
                .min(self.color_attachment.width() as f32 - 1.0);
            let aabb_max_y = vertices
                .iter()
                .fold(std::f32::MIN, |max, v| {
                    if v.position.y > max {
                        v.position.y
                    } else {
                        max
                    }
                })
                .floor()
                .min(self.color_attachment.height() as f32 - 1.0);
            let aabb_min = math::Vec2::new(aabb_min_x, aabb_min_y);
            let aabb_max = math::Vec2::new(aabb_max_x, aabb_max_y);

            // 5. walk through all pixel in AABB and set color
            for x in aabb_min.x as u32..=aabb_max.x as u32 {
                for y in aabb_min.y as u32..=aabb_max.y as u32 {
                    let berycentric = math::Berycentric::new(
                        &math::Vec2::new(x as f32, y as f32),
                        &vertices.map(|v| math::Vec2::new(v.position.x, v.position.y)),
                    );
                    if berycentric.is_valid() {
                        // 6. attributes interpolation
                        let mut color = vertices[0].attributes.vec4[ATTR_COLOR]
                            * berycentric.alpha()
                            + vertices[1].attributes.vec4[ATTR_COLOR] * berycentric.beta()
                            + vertices[2].attributes.vec4[ATTR_COLOR] * berycentric.gamma();
                        match texture {
                            Some(texture) => {
                                let texcoord = vertices[0].attributes.vec2[ATTR_TEXCOORD]
                                    * berycentric.alpha()
                                    + vertices[1].attributes.vec2[ATTR_TEXCOORD]
                                        * berycentric.beta()
                                    + vertices[2].attributes.vec2[ATTR_TEXCOORD]
                                        * berycentric.gamma();
                                color *= texture_sample(texture, &texcoord);
                            }
                            None => {}
                        }
                        self.color_attachment.set(x, y, &color);
                    }
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