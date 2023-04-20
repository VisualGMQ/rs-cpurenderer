use crate::{
    camera,
    image::{ColorAttachment, DepthAttachment},
    math,
    renderer::{self, should_cull, FaceCull, FrontFace},
    scanline::Trapezoid,
    scanline::*,
    shader::{self, Shader, Uniforms, Vertex},
    texture::TextureStorage,
};

pub struct Renderer {
    color_attachment: ColorAttachment,
    depth_attachment: DepthAttachment,
    camera: camera::Camera,
    viewport: renderer::Viewport,
    shader: Shader,
    uniforms: Uniforms,
    front_face: FrontFace,
    cull: FaceCull,

    cliped_triangles: Vec<Vertex>,
}

impl renderer::RendererInterface for Renderer {
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
        texture_storage: &TextureStorage,
    ) {
        let count = if self.cliped_triangles.is_empty() {
            vertices
        } else {
            &self.cliped_triangles
        }
        .len()
            / 3;
        for i in 0..count as usize {
            // convert 3D coordination to Homogeneous coordinates
            let mut vertices = {
                let vertices = if self.cliped_triangles.is_empty() {
                    vertices
                } else {
                    &self.cliped_triangles
                };

                [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]]
            };

            // call vertex changing function to change vertex position and set attribtues
            for v in &mut vertices {
                *v = self
                    .shader
                    .call_vertex_changing(v, &self.uniforms, texture_storage);
            }

            // Model View transform
            for v in &mut vertices {
                v.position = *self.camera.view_mat() * *model * v.position;
            }

            // project transform
            for v in &mut vertices {
                v.position = *self.camera.get_frustum().get_mat() * v.position;
            }

            // save truely z into v.position.z
            for v in &mut vertices {
                v.position.z = -v.position.w * self.camera.get_frustum().near();
            }

            // perspective divide
            for v in &mut vertices {
                v.position.x /= v.position.w;
                v.position.y /= v.position.w;
                v.position.w = 1.0;
            }

            // Face Cull
            if should_cull(
                &vertices.map(|v| v.position.truncated_to_vec3()),
                &-*math::Vec3::z_axis(),
                self.front_face,
                self.cull,
            ) {
                continue;
            }

            // Viewport transform
            for v in &mut vertices {
                v.position.x = (v.position.x + 1.0) * 0.5 * (self.viewport.w as f32 - 1.0)
                    + self.viewport.x as f32;
                v.position.y = self.viewport.h as f32
                    - (v.position.y + 1.0) * 0.5 * (self.viewport.h as f32 - 1.0)
                    + self.viewport.y as f32;
            }

            // split triangle into trapeziods
            let [trap1, trap2] = &mut Trapezoid::from_triangle(&vertices);

            // rasterization trapeziods
            if let Some(trap) = trap1 {
                self.draw_trapezoid(trap, texture_storage);
            }
            if let Some(trap) = trap2 {
                self.draw_trapezoid(trap, texture_storage);
            }
        }
    }

    fn get_shader(&mut self) -> &mut shader::Shader {
        &mut self.shader
    }

    fn get_uniforms(&mut self) -> &mut Uniforms {
        &mut self.uniforms
    }

    fn clear_depth(&mut self) {
        self.depth_attachment.clear(f32::MIN);
    }

    fn get_camera(&mut self) -> &mut camera::Camera {
        &mut self.camera
    }

    fn set_camera(&mut self, camera: camera::Camera) {
        self.camera = camera;
    }

    fn set_front_face(&mut self, front_face: FrontFace) {
        self.front_face = front_face;
    }

    fn get_front_face(&self) -> FrontFace {
        self.front_face
    }

    fn set_face_cull(&mut self, cull: FaceCull) {
        self.cull = cull;
    }

    fn get_face_cull(&self) -> FaceCull {
        self.cull
    }
}

impl Renderer {
    pub fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            depth_attachment: DepthAttachment::new(w, h),
            camera,
            viewport: renderer::Viewport { x: 0, y: 0, w, h },
            shader: Default::default(),
            uniforms: Default::default(),
            front_face: FrontFace::CW,
            cull: FaceCull::None,
            cliped_triangles: Vec::new(),
        }
    }

    fn draw_trapezoid(&mut self, trap: &mut Trapezoid, texture_storage: &TextureStorage) {
        let top = (trap.top.ceil().max(0.0)) as i32;
        let bottom =
            (trap.bottom.ceil()).min(self.color_attachment.height() as f32 - 1.0) as i32 - 1;
        let mut y = top as f32;

        shader::vertex_rhw_init(&mut trap.left.v1);
        shader::vertex_rhw_init(&mut trap.left.v2);
        shader::vertex_rhw_init(&mut trap.right.v1);
        shader::vertex_rhw_init(&mut trap.right.v2);

        while y <= bottom as f32 {
            let mut scanline = Scanline::from_trapezoid(trap, y);
            self.draw_scanline(&mut scanline, texture_storage);
            y += 1.0;
        }
    }

    fn draw_scanline(&mut self, scanline: &mut Scanline, texture_storage: &TextureStorage) {
        let vertex = &mut scanline.vertex;
        let y = scanline.y as u32;
        while scanline.width > 0.0 {
            let rhw = vertex.position.z;
            let z = 1.0 / rhw;

            let x = vertex.position.x;

            if x >= 0.0 && x < self.color_attachment.width() as f32 {
                let x = x as u32;
                if self.depth_attachment.get(x, y) <= z {
                    let mut attr = vertex.attributes;
                    shader::attributes_foreach(&mut attr, |value| value / rhw);
                    // call pixel shading function to get shading color
                    let color =
                        self.shader
                            .call_pixel_shading(&attr, &self.uniforms, texture_storage);
                    self.color_attachment.set(x, y, &color);
                    self.depth_attachment.set(x, y, z);
                }
            }

            scanline.width -= 1.0;
            vertex.position += scanline.step.position;
            vertex.attributes = shader::interp_attributes(
                &vertex.attributes,
                &scanline.step.attributes,
                |value1, value2, _| value1 + value2,
                0.0,
            );
        }
    }
}
