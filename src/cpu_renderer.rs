use crate::{
    camera,
    image::ColorAttachment,
    math,
    renderer::{self},
    scanline::Trapezoid,
    scanline::*,
    shader::{self, Shader, Uniforms, Vertex}, texture::TextureStorage,
};

pub struct Renderer {
    color_attachment: ColorAttachment,
    camera: camera::Camera,
    viewport: renderer::Viewport,
    shader: Shader,
    uniforms: Uniforms,
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
        count: u32,
        texture_storage: &TextureStorage,
    ) {
        for i in 0..count as usize {
            // convert 3D coordination to Homogeneous coordinates
            let mut vertices = [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]];

            // call vertex changing function to change vertex position and set attribtues
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
}

impl Renderer {
    pub fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            camera,
            viewport: renderer::Viewport { x: 0, y: 0, w, h },
            shader: Default::default(),
            uniforms: Default::default(),
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
            let mut scanline = Scanline::from_trapezoid(&trap, y);
            self.draw_scanline(&mut scanline, texture_storage);
            y += 1.0;
        }
    }

    fn draw_scanline(&mut self, scanline: &mut Scanline, texture_storage: &TextureStorage) {
        let vertex = &mut scanline.vertex;
        let y = scanline.y as u32;
        while scanline.width > 0.0 {
            let rhw = vertex.position.z;

            let x = vertex.position.x;

            if x >= 0.0 && x < self.color_attachment.width() as f32 {
                let mut attr = vertex.attributes;
                shader::attributes_foreach(&mut attr, |value| value / rhw);
                // call pixel shading function to get shading color
                let color = self.shader.call_pixel_shading(&attr, &self.uniforms, texture_storage);
                self.color_attachment.set(x as u32, y, &color);
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
