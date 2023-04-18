use fltk::app::set_visual;
use fltk::enums::Mode;
use fltk::{prelude::*, window::Window};
use rs_cpurenderer::renderer::texture_sample;
use rs_cpurenderer::shader::{Attributes, Vertex};
use rs_cpurenderer::texture::TextureStorage;
use rs_cpurenderer::{camera, cpu_renderer, gpu_renderer, math, renderer::RendererInterface};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 720;

// attribute location
const ATTR_COLOR: usize = 0;
const ATTR_TEXCOORD: usize = 1;

// uniform location
const UNIFORM_TEXTURE: u32 = 0;

fn swap_context(renderer: &mut Box<dyn RendererInterface>) {
    let result = renderer.get_rendered_image();
    fltk::draw::draw_image(
        result,
        0,
        0,
        renderer.get_canva_width() as i32,
        renderer.get_canva_height() as i32,
        fltk::enums::ColorDepth::Rgb8,
    )
    .unwrap();
}

pub fn create_renderer(w: u32, h: u32, camera: camera::Camera) -> Box<dyn RendererInterface> {
    if cfg!(feature = "cpu") {
        println!("use cpu renderer");
        Box::new(cpu_renderer::Renderer::new(w, h, camera))
    } else {
        println!("use gpu renderer");
        Box::new(gpu_renderer::Renderer::new(w, h, camera))
    }
}

fn main() {
    let camera = camera::Camera::new(
        1.0,
        1000.0,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        30f32.to_radians(),
    );
    let mut renderer = create_renderer(WINDOW_WIDTH, WINDOW_HEIGHT, camera);

    let mut texture_storage = TextureStorage::default();
    let texture_id = texture_storage.load("./resources/plane/pic.jpg").unwrap();
    renderer.get_uniforms().texture.insert(UNIFORM_TEXTURE, texture_id);

    // data prepare
    let mut attr1 = Attributes::default();
    let mut attr2 = Attributes::default();
    let mut attr3 = Attributes::default();
    let mut attr4 = Attributes::default();
    attr1.set_vec4(ATTR_COLOR, math::Vec4::new(1.0, 1.0, 1.0, 1.0));
    attr2.set_vec4(ATTR_COLOR, math::Vec4::new(1.0, 1.0, 1.0, 1.0));
    attr3.set_vec4(ATTR_COLOR, math::Vec4::new(1.0, 1.0, 1.0, 1.0));
    attr4.set_vec4(ATTR_COLOR, math::Vec4::new(1.0, 1.0, 1.0, 1.0));
    attr1.set_vec2(ATTR_TEXCOORD, math::Vec2::new(0.0, 1.0));
    attr2.set_vec2(ATTR_TEXCOORD, math::Vec2::new(1.0, 1.0));
    attr3.set_vec2(ATTR_TEXCOORD, math::Vec2::new(0.0, 0.0));
    attr4.set_vec2(ATTR_TEXCOORD, math::Vec2::new(1.0, 0.0));

    // vertex changing shader(as vertex shader in OpenGL)
    renderer.get_shader().vertex_changing = Box::new(|vertex, _, _| {
        *vertex
    });

    // pixel shading shader(as fragment shader in OpenGL)
    renderer.get_shader().pixel_shading = Box::new(|attr, uniforms, texture_storage| {
        let frag_color = attr.vec4[ATTR_COLOR];
        let texcoord = attr.vec2[ATTR_TEXCOORD];
        let texture = texture_storage.get(uniforms.texture[&UNIFORM_TEXTURE]).unwrap();
        let texture_color = texture_sample(texture, &texcoord);
        texture_color * frag_color
    });

    let app = fltk::app::App::default();
    let mut wind = Window::new(
        100,
        100,
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32,
        "sandbox",
    );

    let mut rotation = 0.0f32;


    wind.draw(move |_| {
        renderer.clear(&math::Vec4::new(0.2, 0.2, 0.2, 1.0));

        let model = math::create_translate(&math::Vec3::new(0.0, 0.0, -4.0))
            * math::create_eular_rotate_y(rotation.to_radians());

        let vertices = [
            Vertex::new(math::Vec3::new(-1.0, 1.0, 0.0), attr1),
            Vertex::new(math::Vec3::new(1.0, 1.0, 0.0), attr2),
            Vertex::new(math::Vec3::new(-1.0, -1.0, 0.0), attr3),
            Vertex::new(math::Vec3::new(1.0, 1.0, 0.0), attr2),
            Vertex::new(math::Vec3::new(-1.0, -1.0, 0.0), attr3),
            Vertex::new(math::Vec3::new(1.0, -1.0, 0.0), attr4),
        ];

        renderer.draw_triangle(&model, &vertices, 2, &texture_storage);

        rotation += 1.0;

        swap_context(&mut renderer);
    });

    wind.handle(move |_, _| false);

    wind.end();
    set_visual(Mode::Rgb).unwrap();
    wind.show();

    fltk::app::add_idle3(move |_| {
        wind.redraw();
    });

    app.run().unwrap();
}
