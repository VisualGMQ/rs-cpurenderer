use crate::{renderer, image::ColorAttachment, camera, math, scanline::Trapezoid, scanline::*};

pub struct Renderer {
    color_attachment: ColorAttachment,
    camera: camera::Camera,
    viewport: renderer::Viewport,
}

impl renderer::RendererInterface for Renderer {
    fn new(w: u32, h: u32, camera: camera::Camera) -> Self {
        Self {
            color_attachment: ColorAttachment::new(w, h),
            camera,
            viewport: renderer::Viewport { x: 0, y: 0, w, h },
        }
    }

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


        // 4. split triangle into trapeziods
        let [trap1, trap2] = &mut Trapezoid::from_triangle(&vertices);

        // 6. rasterization trapeziods
        if let Some(trap) = trap1 {
            self.draw_trapezoid(trap, color);
        }
        if let Some(trap) = trap2 {
            self.draw_trapezoid(trap, color);
        }


        for i in 0..vertices.len() {
            let p1 = &vertices[i];
            let p2 = &vertices[(i + 1) % vertices.len()];

            renderer::bresenham::draw_line(&mut self.color_attachment, p1, p2, color);
        }
    }
}

impl Renderer {
    fn draw_trapezoid(&mut self, trap: &Trapezoid, color: &math::Vec4) {
        let top = (trap.top.ceil().max(0.0)) as i32;
        let bottom =
            (trap.bottom.ceil()).min(self.color_attachment.height() as f32 - 1.0) as i32 - 1;
        let mut y = top as f32;

        while y <= bottom as f32 {
            let mut scanline = Scanline::from_trapezoid(&trap, y);
            self.draw_scanline(&mut scanline, color);
            y += 1.0;
        }
    }

    fn draw_scanline(&mut self, scanline: &mut Scanline, color: &math::Vec4) {
        let vertex = &mut scanline.vertex;
        let y = scanline.y as u32;
        while scanline.width > 0.0 {
            let x = vertex.x;

            if x >= 0.0 && x < self.color_attachment.width() as f32 {
                let x = x as u32;
                self.color_attachment.set(x, y, &color)
            }

            scanline.width -= 1.0;
            *vertex += scanline.step;
        }
    }
}
