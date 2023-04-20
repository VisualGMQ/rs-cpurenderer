use crate::{
    camera,
    image::{ColorAttachment, DepthAttachment},
    line::Line,
    math,
    renderer::{self, rasterize_line, should_cull, FaceCull, FrontFace},
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
    enable_framework: bool,
}

enum RasterizeResult {
    Ok,
    Discard,
    GenerateNewFace,
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
        for i in 0..vertices.len() / 3_usize {
            // convert 3D coordination to Homogeneous coordinates
            let vertices = [vertices[i * 3], vertices[1 + i * 3], vertices[2 + i * 3]];

            match self.rasterize_trianlge(model, vertices, texture_storage) {
                RasterizeResult::Ok | RasterizeResult::Discard => {}
                RasterizeResult::GenerateNewFace => {
                    for i in 0..self.cliped_triangles.len() / 3 {
                        let vertices = [
                            self.cliped_triangles[i * 3],
                            self.cliped_triangles[1 + i * 3],
                            self.cliped_triangles[2 + i * 3],
                        ];
                        match self.rasterize_trianlge(model, vertices, texture_storage) {
                            RasterizeResult::Ok => {}
                            RasterizeResult::Discard | RasterizeResult::GenerateNewFace => {
                                panic!("discard or generate new face from clipped face")
                            }
                        }
                        self.cliped_triangles.clear();
                    }
                }
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

    fn enable_framework(&mut self) {
        self.enable_framework = true;
    }

    fn disable_framework(&mut self) {
        self.enable_framework = false;
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
            enable_framework: false,
        }
    }

    fn rasterize_trianlge(
        &mut self,
        model: &math::Mat4,
        mut vertices: [Vertex; 3],
        texture_storage: &TextureStorage,
    ) -> RasterizeResult {
        // call vertex changing function to change vertex position and set attribtues
        for v in &mut vertices {
            *v = self
                .shader
                .call_vertex_changing(v, &self.uniforms, texture_storage);
        }

        // Model transform
        for v in &mut vertices {
            v.position = *model * v.position;
        }

        // Face Cull
        if should_cull(
            &vertices.map(|v| v.position.truncated_to_vec3()),
            self.camera.view_dir(),
            self.front_face,
            self.cull,
        ) {
            return RasterizeResult::Discard;
        }

        // view transform
        for v in &mut vertices {
            v.position = *self.camera.view_mat() * v.position;
        }

        // frustum clip
        if vertices.iter().all(|v| {
            !self
                .camera
                .get_frustum()
                .contain(&v.position.truncated_to_vec3())
        }) {
            return RasterizeResult::Discard;
        }

        // near plane clip
        if vertices
            .iter()
            .any(|v| v.position.z > self.camera.get_frustum().near())
        {
            let (face1, face2) =
                crate::scanline::near_plane_clip(&vertices, self.camera.get_frustum().near());
            self.cliped_triangles.extend(face1.iter());
            if let Some(face) = face2 {
                self.cliped_triangles.extend(face.iter());
            }
            return RasterizeResult::GenerateNewFace;
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

        if self.enable_framework {
            // draw line framework
            for i in 0..3 {
                let mut v1 = vertices[i];
                let mut v2 = vertices[(i + 1) % 3];
                v1.position.z = 1.0 / v1.position.z;
                v2.position.z = 1.0 / v2.position.z;

                rasterize_line(
                    &Line::new(v1, v2),
                    &self.shader.pixel_shading,
                    &self.uniforms,
                    texture_storage,
                    &mut self.color_attachment,
                    &mut self.depth_attachment,
                );
            }
        } else {
            // rasterization triangle
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

        RasterizeResult::Ok
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
