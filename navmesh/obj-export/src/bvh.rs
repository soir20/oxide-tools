use std::collections::HashMap;

use bvh::{
    aabb::{Aabb, Bounded},
    bounding_hierarchy::BHShape,
    bvh::Bvh,
};
use serde::Serialize;

fn vertex_from_index(vertices: &[[f32; 3]], index: u16) -> [f32; 3] {
    let index = usize::from(index);
    vertices[index]
}

fn triangle_to_aabb(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> Aabb<f32, 3> {
    Aabb::with_bounds(
        [
            v1[0].min(v2[0]).min(v3[0]),
            v1[1].min(v2[1]).min(v3[1]),
            v1[2].min(v2[2]).min(v3[2]),
        ]
        .into(),
        [
            v1[0].max(v2[0]).max(v3[0]),
            v1[1].max(v2[1]).max(v3[1]),
            v1[2].max(v2[2]).max(v3[2]),
        ]
        .into(),
    )
}

struct Triangle {
    aabb: Aabb<f32, 3>,
    node_index: usize,
}

impl Triangle {
    pub fn from_vertices(vertices: &[[f32; 3]], triangle: [u16; 3]) -> Self {
        let v1 = vertex_from_index(vertices, triangle[0]);
        let v2 = vertex_from_index(vertices, triangle[1]);
        let v3 = vertex_from_index(vertices, triangle[2]);
        Triangle {
            aabb: triangle_to_aabb(v1, v2, v3),
            node_index: 0,
        }
    }
}

impl Bounded<f32, 3> for Triangle {
    fn aabb(&self) -> Aabb<f32, 3> {
        self.aabb
    }
}

impl BHShape<f32, 3> for Triangle {
    fn set_bh_node_index(&mut self, node_index: usize) {
        self.node_index = node_index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}

pub fn generate_bvh(vertices: &[[f32; 3]], triangles: &[[u16; 3]]) -> Bvh<f32, 3> {
    let mut shapes: Vec<Triangle> = triangles
        .iter()
        .map(|triangle| Triangle::from_vertices(vertices, *triangle))
        .collect();
    Bvh::build(&mut shapes)
}

#[derive(Serialize)]
pub struct BvhInstance {
    pub name: String,
    pub pos: [f32; 3],
    pub rot: [f32; 3],
}

#[derive(Serialize)]
pub struct BvhTemplate {
    pub bvh: Bvh<f32, 3>,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<[u16; 3]>,
}

#[derive(Serialize)]
pub struct BvhFile {
    pub templates: HashMap<String, BvhTemplate>,
    pub instances: Vec<BvhInstance>,
}
