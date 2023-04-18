use crate::{
    camera,
    image::ColorAttachment,
    math::{self, Berycentric},
    renderer::*,
    shader::*, texture::{TextureStorage, self},
};

pub struct Renderer {
    color_attachment: ColorAttachment,
    camera: camera::Camera,
    viewport: Viewport,
    shader: Shader,
    uniforms: Uniforms,
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
        texture_storage: &TextureStorage
    ) {
        for i in 0..count as usize {
            // convert 3D coordination to Homogeneous coordinates
            let mut vertices = [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]];

            for v in &mut vertices {
                *v = self.shader.call_vertex_changing(&v, &self.uniforms, texture_storage);
            }

            // MV transform
            for v in &mut vertices {
                v.position = *model * v.position;
            }

            // project transform
            for v in &mut vertices {
                v.position = *self.camera.get_frustum().get_mat() * v.position;
            }

            // set truely z
            for v in &mut vertices {
                v.position.z = -v.position.w;
            }

            // perspective divide
            for v in &mut vertices {
                v.position.x /= v.position.w;
                v.position.y /= v.position.w;
            }

            // Viewport transform
            for v in &mut vertices {
                v.position.x = (v.position.x + 1.0) * 0.5 * (self.viewport.w as f32 - 1.0)
                    + self.viewport.x as f32;
                v.position.y = self.viewport.h as f32
                    - (v.position.y + 1.0) * 0.5 * (self.viewport.h as f32 - 1.0)
                    + self.viewport.y as f32;
            }

            // find AABB for triangle
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

            // walk through all pixel in AABB and set color
            for x in aabb_min.x as u32..=aabb_max.x as u32 {
                for y in aabb_min.y as u32..=aabb_max.y as u32 {
                    let berycentric = math::Berycentric::new(
                        &math::Vec2::new(x as f32, y as f32),
                        &vertices.map(|v| math::Vec2::new(v.position.x, v.position.y)),
                    );
                    if berycentric.is_valid() {
                        // attributes interpolation and perspective correct
                        let inv_z = berycentric.alpha() / vertices[0].position.z
                            + berycentric.beta() / vertices[1].position.z
                            + berycentric.gamma() / vertices[2].position.z;
                        let z = 1.0 / inv_z;
                        let attr = get_corrected_attribute(z, &vertices, &berycentric);
                        //  call pixel shading function to get pixel color
                        let color = self.shader.call_pixel_shading(&attr, &self.uniforms, texture_storage);
                        self.color_attachment.set(x, y, &color);
                    }
                }
            }
        }
    }

    fn get_shader(&mut self) -> &mut Shader {
        &mut self.shader
    }

    fn get_uniforms(&mut self) -> &mut Uniforms {
        &mut self.uniforms
    }
}

#[rustfmt::skip]
fn get_corrected_attribute(z: f32, vertices: &[Vertex; 3], berycentric: &Berycentric) -> Attributes {
    let mut attr = Attributes::default();
    for i in 0..attr.float.len() {
        attr.float[i] = (vertices[0].attributes.float[i] * berycentric.alpha() / vertices[0].position.z +
                         vertices[1].attributes.float[i] * berycentric.beta() / vertices[1].position.z +
                         vertices[2].attributes.float[i] * berycentric.gamma() / vertices[2].position.z) * z;
        attr.vec2[i] = (vertices[0].attributes.vec2[i] * berycentric.alpha() / vertices[0].position.z +
                        vertices[1].attributes.vec2[i] * berycentric.beta() / vertices[1].position.z +
                        vertices[2].attributes.vec2[i] * berycentric.gamma() / vertices[2].position.z) * z;
        attr.vec3[i] = (vertices[0].attributes.vec3[i] * berycentric.alpha() / vertices[0].position.z +
                        vertices[1].attributes.vec3[i] * berycentric.beta() / vertices[1].position.z +
                        vertices[2].attributes.vec3[i] * berycentric.gamma() / vertices[2].position.z) * z;
        attr.vec4[i] = (vertices[0].attributes.vec4[i] * berycentric.alpha() / vertices[0].position.z +
                        vertices[1].attributes.vec4[i] * berycentric.beta() / vertices[1].position.z +
                        vertices[2].attributes.vec4[i] * berycentric.gamma() / vertices[2].position.z) * z;
    }
    attr
}

impl Renderer {
    pub fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            camera,
            viewport: Viewport { x: 0, y: 0, w, h },
            shader: Default::default(),
            uniforms: Default::default(),
        }
    }
}
