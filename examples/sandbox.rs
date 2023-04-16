use fltk::app::set_visual;
use fltk::enums::Mode;
use fltk::{prelude::*, window::Window};
use rs_cpurenderer::{camera, cpu_renderer, gpu_renderer, math, renderer::RendererInterface};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 720;

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
    if cfg!(feature="cpu") {
        println!("use cpu renderer");
        Box::new(cpu_renderer::Renderer::new(
            w,
            h,
            camera,
        ))
    } else {
        println!("use gpu renderer");
        Box::new(gpu_renderer::Renderer::new(
            w,
            h,
            camera,
        ))
    }
}

fn main() {
    let camera = camera::Camera::new(
        1.0,
        1000.0,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        30f32.to_radians(),
    );
    let mut renderer = create_renderer(
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        camera,
    );

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

        let color = math::Vec4::new(0.0, 1.0, 0.0, 1.0);

        let model = math::create_translate(&math::Vec3::new(0.0, 0.0, -4.0))
            * math::create_eular_rotate_y(rotation.to_radians());

        let vertices = [
            math::Vec3::new(-1.0, 1.0, 0.0),
            math::Vec3::new(1.0, 1.0, 0.0),
            math::Vec3::new(0.0, -1.0, 0.0),
        ];

        renderer.draw_triangle(&model, &vertices, &color);

        rotation += 1.0;

        swap_context(&mut renderer);
    });

    wind.handle(move |_, event| false);

    wind.end();
    set_visual(Mode::Rgb).unwrap();
    wind.show();

    fltk::app::add_idle3(move |_| {
        wind.redraw();
    });

    app.run().unwrap();
}
