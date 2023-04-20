use crate::shader::{interp_attributes, Vertex};

pub(crate) struct Line {
    pub start: Vertex,
    pub end: Vertex,
    pub step: Vertex,
}

impl Line {
    pub fn new(start: Vertex, end: Vertex) -> Self {
        let dx = (end.position.x - start.position.x).abs();
        let dy = (end.position.y - start.position.y).abs();
        let t = if dx >= dy {
            1.0 / (end.position.x - start.position.x).abs()
        } else {
            1.0 / (end.position.y - start.position.y).abs()
        };

        Self {
            start,
            end,
            step: Vertex {
                attributes: interp_attributes(
                    &start.attributes,
                    &end.attributes,
                    |value1, value2, t| (value2 - value1) * t,
                    t,
                ),
                position: (end.position - start.position) * t,
            },
        }
    }
}
