use crate::math;

const MAX_ATTRIBUTES_NUM: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Attributes {
    pub float: [f32; MAX_ATTRIBUTES_NUM],
    pub vec2: [math::Vec2; MAX_ATTRIBUTES_NUM],
    pub vec3: [math::Vec3; MAX_ATTRIBUTES_NUM],
    pub vec4: [math::Vec4; MAX_ATTRIBUTES_NUM],
}

impl Attributes {
    pub fn set_float(&mut self, location: usize, value: f32) {
        self.float[location] = value;
    }

    pub fn set_vec2(&mut self, location: usize, value: math::Vec2) {
        self.vec2[location] = value;
    }

    pub fn set_vec3(&mut self, location: usize, value: math::Vec3) {
        self.vec3[location] = value;
    }

    pub fn set_vec4(&mut self, location: usize, value: math::Vec4) {
        self.vec4[location] = value;
    }
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            float: [0.0; MAX_ATTRIBUTES_NUM],
            vec2: [math::Vec2::zero(); MAX_ATTRIBUTES_NUM],
            vec3: [math::Vec3::zero(); MAX_ATTRIBUTES_NUM],
            vec4: [math::Vec4::zero(); MAX_ATTRIBUTES_NUM],
        }
    }
}

pub fn lerp_vertex(start: &Vertex, end: &Vertex, t: f32) -> Vertex {
    let position = start.position + (end.position - start.position) * t;
    let attributes = interp_attributes(&start.attributes, &end.attributes, math::lerp, t);

    Vertex {
        position,
        attributes,
    }
}

pub fn vertex_rhw_init(vertex: &mut Vertex) {
    let rhw_z = 1.0 / vertex.position.z;
    vertex.position.z = rhw_z;

    attributes_foreach(&mut vertex.attributes, |value| value * rhw_z);
}

pub fn interp_attributes<F>(attr1: &Attributes, attr2: &Attributes, f: F, t: f32) -> Attributes
where
    F: Fn(f32, f32, f32) -> f32,
{
    let mut attributes = Attributes::default();

    for index in 0..MAX_ATTRIBUTES_NUM {
        attributes.set_float(index, f(attr1.float[index], attr2.float[index], t));
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value1 = attr1.vec2[index];
        let value2 = attr2.vec2[index];
        attributes.set_vec2(
            index,
            math::Vec2::new(f(value1.x, value2.x, t), f(value1.y, value2.y, t)),
        );
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value1 = attr1.vec3[index];
        let value2 = attr2.vec3[index];
        attributes.set_vec3(
            index,
            math::Vec3::new(
                f(value1.x, value2.x, t),
                f(value1.y, value2.y, t),
                f(value1.z, value2.z, t),
            ),
        );
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value1 = attr1.vec4[index];
        let value2 = attr2.vec4[index];
        attributes.set_vec4(
            index,
            math::Vec4::new(
                f(value1.x, value2.x, t),
                f(value1.y, value2.y, t),
                f(value1.z, value2.z, t),
                f(value1.w, value2.w, t),
            ),
        );
    }

    attributes
}

pub fn attributes_foreach<F>(attr: &mut Attributes, f: F)
where
    F: Fn(f32) -> f32,
{
    for index in 0..MAX_ATTRIBUTES_NUM {
        attr.set_float(index, f(attr.float[index]));
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value = attr.vec2[index];
        attr.set_vec2(index, math::Vec2::new(f(value.x), f(value.y)));
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value = attr.vec3[index];
        attr.set_vec3(index, math::Vec3::new(f(value.x), f(value.y), f(value.z)));
    }

    for index in 0..MAX_ATTRIBUTES_NUM {
        let value = attr.vec4[index];
        attr.set_vec4(
            index,
            math::Vec4::new(f(value.x), f(value.y), f(value.z), f(value.w)),
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: math::Vec4,
    pub attributes: Attributes,
}

impl Vertex {
    pub fn new(position: math::Vec3, attributes: Attributes) -> Self {
        Self {
            position: math::Vec4::from_vec3(&position, 1.0),
            attributes,
        }
    }
}
