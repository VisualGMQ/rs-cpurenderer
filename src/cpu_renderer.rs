use crate::{
    camera,
    image::ColorAttachment,
    math,
    renderer::{self, texture_sample, ATTR_COLOR, ATTR_TEXCOORD},
    scanline::Trapezoid,
    scanline::*,
    vertex::{self, Vertex},
};

pub struct Renderer {
    color_attachment: ColorAttachment,
    camera: camera::Camera,
    viewport: renderer::Viewport,
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
        texture: Option<&image::DynamicImage>,
    ) {
        for i in 0..count as usize {
            // 1. convert 3D coordination to Homogeneous coordinates
            let mut vertices = [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]];

            // 2. MV transform
            for v in &mut vertices {
                v.position = *model * v.position;
            }

            // 3. project transform
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

            // 4. Viewport transform
            for v in &mut vertices {
                v.position.x = (v.position.x + 1.0) * 0.5 * (self.viewport.w as f32 - 1.0)
                    + self.viewport.x as f32;
                v.position.y = self.viewport.h as f32
                    - (v.position.y + 1.0) * 0.5 * (self.viewport.h as f32 - 1.0)
                    + self.viewport.y as f32;
            }

            // 5. split triangle into trapeziods
            let [trap1, trap2] = &mut Trapezoid::from_triangle(&vertices);

            // 6. rasterization trapeziods
            if let Some(trap) = trap1 {
                self.draw_trapezoid(trap, texture);
            }
            if let Some(trap) = trap2 {
                self.draw_trapezoid(trap, texture);
            }
        }
    }
}

impl Renderer {
    pub fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            camera,
            viewport: renderer::Viewport { x: 0, y: 0, w, h },
        }
    }

    fn draw_trapezoid(&mut self, trap: &mut Trapezoid, texture: Option<&image::DynamicImage>) {
        let top = (trap.top.ceil().max(0.0)) as i32;
        let bottom =
            (trap.bottom.ceil()).min(self.color_attachment.height() as f32 - 1.0) as i32 - 1;
        let mut y = top as f32;

        vertex::vertex_rhw_init(&mut trap.left.v1);
        vertex::vertex_rhw_init(&mut trap.left.v2);
        vertex::vertex_rhw_init(&mut trap.right.v1);
        vertex::vertex_rhw_init(&mut trap.right.v2);

        while y <= bottom as f32 {
            let mut scanline = Scanline::from_trapezoid(&trap, y);
            self.draw_scanline(&mut scanline, texture);
            y += 1.0;
        }
    }

    fn draw_scanline(&mut self, scanline: &mut Scanline, texture: Option<&image::DynamicImage>) {
        let vertex = &mut scanline.vertex;
        let y = scanline.y as u32;
        while scanline.width > 0.0 {
            let rhw = vertex.position.z;

            let x = vertex.position.x;

            if x >= 0.0 && x < self.color_attachment.width() as f32 {
                let mut attr = vertex.attributes;
                vertex::attributes_foreach(&mut attr, |value| value / rhw);

                let texcoord = attr.vec2[ATTR_TEXCOORD];

                let color = attr.vec4[ATTR_COLOR]
                    * match texture {
                        Some(texture) => texture_sample(texture, &texcoord),
                        None => math::Vec4::new(1.0, 1.0, 1.0, 1.0),
                    };
                self.color_attachment.set(x as u32, y, &color);
            }

            scanline.width -= 1.0;
            vertex.position += scanline.step.position;
            vertex.attributes = vertex::interp_attributes(
                &vertex.attributes,
                &scanline.step.attributes,
                |value1, value2, _| value1 + value2,
                0.0,
            );
        }
    }
}
