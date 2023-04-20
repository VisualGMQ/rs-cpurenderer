use crate::camera::Camera;
use crate::image::*;
use crate::math;
use crate::shader::Uniforms;
use crate::shader::{Shader, Vertex};
use crate::texture::Texture;
use crate::texture::TextureStorage;

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum FaceCull {
    Front,
    Back,
    None,
}

#[derive(Clone, Copy, Debug)]
pub enum FrontFace {
    CW,
    CCW,
}

pub trait RendererInterface {
    fn clear(&mut self, color: &math::Vec4);
    fn clear_depth(&mut self);
    fn get_canva_width(&self) -> u32;
    fn get_canva_height(&self) -> u32;
    fn draw_triangle(
        &mut self,
        model: &math::Mat4,
        vertices: &[Vertex],
        texture_storage: &TextureStorage,
    );
    fn get_rendered_image(&self) -> &[u8];
    fn get_shader(&mut self) -> &mut Shader;
    fn get_uniforms(&mut self) -> &mut Uniforms;
    fn get_camera(&mut self) -> &mut Camera;
    fn set_camera(&mut self, camera: Camera);
    fn set_front_face(&mut self, front_face: FrontFace);
    fn get_front_face(&self) -> FrontFace;
    fn set_face_cull(&mut self, cull: FaceCull);
    fn get_face_cull(&self) -> FaceCull;
}

pub fn texture_sample(texture: &Texture, texcoord: &math::Vec2) -> math::Vec4 {
    let x = (texcoord.x * (texture.width() - 1) as f32) as u32;
    let y = (texcoord.y * ((texture.height() - 1) as f32)) as u32;
    texture.get(x, y)
}

pub(crate) fn should_cull(
    positions: &[math::Vec3; 3],
    view_dir: &math::Vec3,
    face: FrontFace,
    cull: FaceCull,
) -> bool {
    let norm = (positions[2] - positions[1]).cross(&(positions[1] - positions[0]));
    let is_front_face = match face {
        FrontFace::CW => norm.dot(view_dir) >= 0.0,
        FrontFace::CCW => norm.dot(view_dir) < 0.0,
    };

    match cull {
        FaceCull::Front => is_front_face,
        FaceCull::Back => !is_front_face,
        FaceCull::None => false,
    }
}

/// [Cohen-Sutherland Algorithm](https://en.wikipedia.org/wiki/Cohen%E2%80%93Sutherland_algorithm)
mod cohen_sutherland {
    use super::math;

    const INSIDE: u8 = 0;
    const LEFT: u8 = 1;
    const RIGHT: u8 = 2;
    const BOTTOM: u8 = 4;
    const TOP: u8 = 8;

    fn compute_outcode(p: &math::Vec2, min: &math::Vec2, max: &math::Vec2) -> u8 {
        (if p.x < min.x {
            LEFT
        } else if p.x > max.x {
            RIGHT
        } else {
            INSIDE
        } | if p.y < min.y {
            BOTTOM
        } else if p.y > max.y {
            TOP
        } else {
            INSIDE
        })
    }

    pub fn cohen_sutherland_line_clip(
        p1: &math::Vec2,
        p2: &math::Vec2,
        rect_min: &math::Vec2,
        rect_max: &math::Vec2,
    ) -> Option<(math::Vec2, math::Vec2)> {
        let mut pt1 = *p1;
        let mut pt2 = *p2;

        let mut outcode1 = compute_outcode(&pt1, rect_min, rect_max);
        let mut outcode2 = compute_outcode(&pt2, rect_min, rect_max);

        loop {
            if outcode1 & outcode2 != 0 {
                return None;
            } else if outcode1 | outcode2 == 0 {
                return Some((pt1, pt2));
            }

            let mut p = math::Vec2::zero();

            let outcode = if outcode2 > outcode1 {
                outcode2
            } else {
                outcode1
            };

            if outcode & TOP != 0 {
                p.x = p1.x + (pt2.x - pt1.x) * (rect_max.y - pt1.y) / (pt2.y - pt1.y);
                p.y = rect_max.y;
            } else if outcode & BOTTOM != 0 {
                p.x = p1.x + (pt2.x - pt1.x) * (rect_min.y - pt1.y) / (pt2.y - pt1.y);
                p.y = rect_min.y;
            } else if outcode & RIGHT != 0 {
                p.y = pt1.y + (pt2.y - pt1.y) * (rect_max.x - pt1.x) / (pt2.x - pt1.x);
                p.x = rect_max.x;
            } else if outcode & LEFT != 0 {
                p.y = pt1.y + (pt2.y - pt1.y) * (rect_min.x - pt1.x) / (pt2.x - pt1.x);
                p.x = rect_min.x;
            }

            if outcode == outcode1 {
                pt1 = p;
                outcode1 = compute_outcode(&pt1, rect_min, rect_max);
            } else {
                pt2 = p;
                outcode2 = compute_outcode(&pt2, rect_min, rect_max);
            }
        }
    }
}

/// [Bresenham Algorithm](https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#)
pub mod bresenham {
    use super::cohen_sutherland;
    use super::math;
    use super::ColorAttachment;

    pub fn draw_line(
        texture: &mut ColorAttachment,
        p1: &math::Vec2,
        p2: &math::Vec2,
        color: &math::Vec4,
    ) {
        let clip_result = cohen_sutherland::cohen_sutherland_line_clip(
            p1,
            p2,
            &math::Vec2::zero(),
            &math::Vec2::new(texture.width() as f32 - 1.0, texture.height() as f32 - 1.0),
        );

        if let Some((p1, p2)) = clip_result {
            draw_line_without_clip(
                texture,
                p1.x as i32,
                p1.y as i32,
                p2.x as i32,
                p2.y as i32,
                color,
            );
        }
    }

    fn draw_line_without_clip(
        texture: &mut ColorAttachment,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        color: &math::Vec4,
    ) {
        let mut dx = (x1 - x0).abs();
        let mut dy = (y1 - y0).abs();
        let mut sx = if x1 >= x0 { 1 } else { -1 };
        let mut sy = if y1 >= y0 { 1 } else { -1 };
        let mut x = x0;
        let mut y = y0;
        let steep = if dx < dy { 1 } else { -1 };

        let final_x = if dx < dy { y1 } else { x1 };

        if dx < dy {
            std::mem::swap(&mut dx, &mut dy);
            std::mem::swap(&mut x, &mut y);
            std::mem::swap(&mut sx, &mut sy);
        }

        let mut e = -dx;
        let step = 2 * dy;
        let desc = -2 * dx;

        while x != final_x {
            if steep > 0 {
                texture.set(y as u32, x as u32, color);
            } else {
                texture.set(x as u32, y as u32, color);
            }

            e += step;
            if e >= 0 {
                y += sy;
                e += desc;
            }
            x += sx;
        }
    }
}
