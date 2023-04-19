use fltk::app::set_visual;
use fltk::enums::Mode;
use fltk::{prelude::*, window::Window};
use rs_cpurenderer::model::{self, Mesh};
use rs_cpurenderer::renderer::texture_sample;
use rs_cpurenderer::shader::{Attributes, Vertex};
use rs_cpurenderer::texture::TextureStorage;
use rs_cpurenderer::{camera, cpu_renderer, gpu_renderer, math, renderer::RendererInterface};

const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 720;

// attribute location
const ATTR_TEXCOORD: usize = 0; // vec2
const ATTR_NORMAL: usize = 0; // vec3

// uniform location
const UNIFORM_TEXTURE: u32 = 0; // vec2
const UNIFORM_COLOR: u32 = 1;   // vec4

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

struct StructedModelData {
    vertices: Vec<Vertex>,
    mtllib: Option<u32>,
    material: Option<String>,
}

fn restruct_model_vertex(meshes: &[Mesh]) -> Vec<StructedModelData> {
    let mut datas = Vec::<StructedModelData>::new();
    for mesh in meshes {
        let mut vertices = Vec::<Vertex>::new();
        for model_vertex in &mesh.vertices {
            let mut attr = Attributes::default();
            attr.set_vec2(ATTR_TEXCOORD, model_vertex.texcoord);
            attr.set_vec3(ATTR_NORMAL, model_vertex.normal);
            let vertex = Vertex::new(model_vertex.position, attr);
            vertices.push(vertex);
        }

        datas.push(StructedModelData {
            vertices,
            mtllib: mesh.mtllib,
            material: mesh.material.clone(),
        });
    }
    datas
}

fn main() {
    let app = fltk::app::App::default();
    let mut wind = Window::new(
        100,
        100,
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32,
        "sandbox",
    );
    let camera = camera::Camera::new(
        1.0,
        1000.0,
        WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        30f32.to_radians(),
    );

    // init renderer and texture storage
    let mut renderer = create_renderer(WINDOW_WIDTH, WINDOW_HEIGHT, camera);
    let mut texture_storage = TextureStorage::default();

    // data prepare, from OBJ model
    const MODEL_ROOT_DIR: &str = "./resources/plane";
    let (meshes, mtllibs) = model::load_from_file(
        &format!("{}/{}", MODEL_ROOT_DIR, "/plane.obj"),
        model::PreOperation::None,
    )
    .unwrap();
    let vertex_datas = restruct_model_vertex(&meshes);

    for mtllib in &mtllibs {
        for (_, material) in mtllib.materials.iter() {
            if let Some(diffuse_map) = &material.texture_maps.diffuse {
                texture_storage
                    .load(&format!("{}/{}", MODEL_ROOT_DIR, diffuse_map), diffuse_map)
                    .unwrap();
            }
        }
    }

    // vertex changing shader(as vertex shader in OpenGL)
    renderer.get_shader().vertex_changing = Box::new(|vertex, _, _| *vertex);

    // pixel shading shader(as fragment shader in OpenGL)
    renderer.get_shader().pixel_shading = Box::new(|attr, uniforms, texture_storage| {
        let mut frag_color = match uniforms.vec4.get(&UNIFORM_COLOR) {
            Some(color) => *color,
            None => math::Vec4::new(1.0, 1.0, 1.0, 1.0),
        };
        let texcoord = attr.vec2[ATTR_TEXCOORD];
        if let Some(texture_id) = uniforms.texture.get(&UNIFORM_TEXTURE) {
            if let Some(texture) = texture_storage.get_by_id(*texture_id) {
                frag_color *= texture_sample(texture, &texcoord);
            }
        }

        frag_color
    });

    let mut rotation = 0.0f32;

    wind.draw(move |_| {
        renderer.clear(&math::Vec4::new(0.2, 0.2, 0.2, 1.0));

        let model = math::create_translate(&math::Vec3::new(0.0, 0.0, -4.0))
            * math::create_eular_rotate_y(rotation.to_radians());

        for data in &vertex_datas {
            // set data into uniform
            let uniforms = renderer.get_uniforms();
            if data.mtllib.is_some() && data.material.is_some() {
                let mtllib = &mtllibs[data.mtllib.unwrap() as usize];
                if let Some(material) = mtllib.materials.get(&data.material.clone().unwrap()) {
                    if let Some(ambient) = material.ambient {
                        uniforms
                            .vec4
                            .insert(UNIFORM_COLOR, math::Vec4::from_vec3(&ambient, 1.0));
                    }
                    if let Some(diffuse_texture) = &material.texture_maps.diffuse {
                        uniforms.texture.insert(
                            UNIFORM_TEXTURE,
                            *texture_storage.get_id(diffuse_texture).unwrap(),
                        );
                    }
                }
            }

            // draw mesh
            renderer.draw_triangle(
                &model,
                &data.vertices,
                (data.vertices.len() / 3).try_into().unwrap(),
                &texture_storage,
            );
        }

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
