use std::sync::Arc;

use crate::{aabb::Aabb, material::Scatter, mikktspace::generate_tangents, triangle::Vertex};
use cgmath::{vec2, vec3, vec4};
use gltf::{
    buffer::{Buffer as GltfBuffer, Data},
    mesh::{Reader, Semantic},
    Document,
};
pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

#[derive(Clone)]
pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}

#[derive(Clone)]
pub struct Primitive {
    index: usize,
    pub bbox: Aabb,
    pub indices: Vec<u32>,
    pub material: Arc<dyn Scatter>,
}

pub unsafe fn create_meshes_from_gltf(document: &Document, buffers: &[Data]) -> Vec<Mesh> {
    let mut meshes: Vec<Mesh> = Vec::new();
    for mesh in document.meshes() {
        let mut primitives: Vec<Primitive> = Vec::new();
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(_accessor) = primitive.get(&Semantic::Positions) {
                let positions = read_positions(&reader);
                let normals = read_normals(&reader);
                let tex_coords_0 = read_tex_coords(&reader, 0);
                let tex_coords_1 = read_tex_coords(&reader, 1);
                let tangents = read_tangents(&reader);
                let colors = read_colors(&reader);

                let mut vertices = positions
                    .iter()
                    .enumerate()
                    .map(|(index, position)| {
                        let position = *position;
                        let normal = *normals.get(index).unwrap_or(&[1.0, 1.0, 1.0]);
                        let tex_coords_0 = *tex_coords_0.get(index).unwrap_or(&[0.0, 0.0]);
                        let _tex_coords_1 = *tex_coords_1.get(index).unwrap_or(&[0.0, 0.0]);
                        let tangent = *tangents.get(index).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);
                        let colors = *colors.get(index).unwrap_or(&[1.0, 1.0, 1.0, 1.0]);

                        Vertex {
                            pos: vec3(position[0], position[1], position[2]),
                            color: vec3(colors[0], colors[1], colors[2]),
                            normal: vec3(normal[0], normal[1], normal[2]),
                            tex_coord: vec2(tex_coords_0[0], tex_coords_0[1]),
                            tangent: vec4(tangent[0], tangent[1], tangent[2], tangent[3]),
                        }
                    })
                    .collect::<Vec<_>>();

                let indices = read_indices(&reader).unwrap();

                if !positions.is_empty()
                    && !normals.is_empty()
                    && !tex_coords_0.is_empty()
                    && tangents.is_empty()
                {
                    generate_tangents(read_indices(&reader).as_deref(), &mut vertices);
                }

                let material = primitive.material().into();

                primitives.push(Primitive {
                    index: 1,
                    indices,
                    material,
                })
            }
        }
        meshes.push(Mesh { primitives })
    }
    meshes
}

fn read_indices<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Option<Vec<u32>>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_indices()
        .map(|indices| indices.into_u32().collect::<Vec<_>>())
}

fn read_positions<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Vec<[f32; 3]>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_positions()
        .expect("Position primitives should be present")
        .collect()
}

fn read_normals<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Vec<[f32; 3]>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_normals()
        .map_or(vec![], |normals| normals.collect())
}

fn read_tex_coords<'a, 's, F>(reader: &Reader<'a, 's, F>, channel: u32) -> Vec<[f32; 2]>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_tex_coords(channel)
        .map_or(vec![], |coords| coords.into_f32().collect())
}

fn read_tangents<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Vec<[f32; 4]>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_tangents()
        .map_or(vec![], |tangents| tangents.collect())
}

fn read_colors<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Vec<[f32; 4]>
where
    F: Clone + Fn(GltfBuffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_colors(0)
        .map_or(vec![], |colors| colors.into_rgba_f32().collect())
}
