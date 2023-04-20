use crate::math;
use crate::obj_loader;
use crate::obj_loader::Mtllib;

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: math::Vec3,
    pub normal: math::Vec3,
    pub texcoord: math::Vec2,
    pub color: math::Vec4,
}

#[derive(Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub name: Option<String>,
    pub mtllib: Option<u32>,
    pub material: Option<String>,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PreOperation {
    None = 0x00,
    RecalcNormal = 0x01,
}

pub fn load_from_file(
    filename: &str,
    pre_operation: PreOperation,
) -> Result<(Vec<Mesh>, Vec<Mtllib>), obj_loader::Error> {
    let mut meshes = vec![];

    let scene = obj_loader::load_from_file(filename)?;

    for model in scene.models {
        let mut mesh = Mesh {
            name: Some(model.name.clone()),
            ..Default::default()
        };
        for face in model.faces {
            for vtx in face.vertices {
                let position = scene.vertices[vtx.vertex as usize];
                let normal = match vtx.normal {
                    None => math::Vec3::zero(),
                    Some(index) => scene.normals[index as usize],
                };
                let texcoord = match vtx.texcoord {
                    None => math::Vec2::zero(),
                    Some(index) => scene.texcoords[index as usize],
                };
                mesh.vertices.push(Vertex {
                    position,
                    normal,
                    texcoord,
                    color: math::Vec4::new(1.0, 1.0, 1.0, 1.0),
                });
            }
        }

        mesh.material = model.material;
        mesh.mtllib = model.mtllib;
        meshes.push(mesh);
    }

    if pre_operation as u8 & PreOperation::RecalcNormal as u8 != 0 {
        for mesh in &mut meshes {
            assert_eq!(mesh.vertices.len() % 3, 0);
            for i in 0..mesh.vertices.len() / 3 {
                let v1 = &mesh.vertices[i * 3];
                let v2 = &mesh.vertices[i * 3 + 1];
                let v3 = &mesh.vertices[i * 3 + 2];
                let norm = (v3.position - v2.position)
                    .cross(&(v2.position - v1.position))
                    .normalize();

                mesh.vertices[i * 3].normal = norm;
                mesh.vertices[i * 3 + 1].normal = norm;
                mesh.vertices[i * 3 + 2].normal = norm;
            }
        }
    }

    Ok((meshes, scene.materials))
}
