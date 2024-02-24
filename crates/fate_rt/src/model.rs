use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Result;
use cgmath::{Point3, Vector2, Vector3};
use std::collections::HashMap;

use crate::aabb::{Aabb, EMPTY};
use crate::bvh::BvhNode;
use crate::hit::{Hit, HitRecord};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Scatter;
use crate::ray::Ray;
use crate::triangle::{Triangle, Vertex};

pub struct Model {
    pub bbox: Aabb,
    pub triangles: HittableList,
    pub mat: Arc<dyn Scatter>,
}

impl Model {
    pub fn new(path: &str, mat: Arc<dyn Scatter>, scale: f32) -> Result<Self> {
        let mut unique_vertices = HashMap::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut bbox: Aabb = EMPTY;
        let mut triangles = HittableList::default();

        if path.ends_with(".obj") {
            let mut reader = BufReader::new(File::open(path)?);

            let (models, _) = tobj::load_obj_buf(
                &mut reader,
                &tobj::LoadOptions {
                    triangulate: true,
                    ..Default::default()
                },
                |_| Ok(Default::default()),
            )?;

            for model in &models {
                let len = model.mesh.indices.len();
                for idx in 0..len {
                    let index = model.mesh.indices[idx];
                    let normal_index = model.mesh.normal_indices[idx] as usize;
                    let pos_offset = (3 * index) as usize;
                    let tex_coord_offset = (2 * index) as usize;

                    let vertex = Vertex {
                        pos: Point3::new(
                            (model.mesh.positions[pos_offset] * scale) as f64,
                            (model.mesh.positions[pos_offset + 1] * scale) as f64,
                            (model.mesh.positions[pos_offset + 2] * scale) as f64,
                        ),
                        color: Vector3::new(1.0, 1.0, 1.0),
                        normal: Vector3::new(
                            model.mesh.normals[normal_index * 3] as f64,
                            model.mesh.normals[normal_index * 3 + 1] as f64,
                            model.mesh.normals[normal_index * 3 + 2] as f64,
                        ),
                        tex_coord: Vector2::new(
                            0.0, //model.mesh.texcoords[tex_coord_offset] as f64,
                            1.0, // - model.mesh.texcoords[tex_coord_offset + 1] as f64,
                        ),
                    };

                    if let Some(index) = unique_vertices.get(&vertex) {
                        indices.push(*index as u32);
                    } else {
                        let index = vertices.len();
                        unique_vertices.insert(vertex, index);
                        vertices.push(vertex);
                        bbox.append(&vertex.pos);
                        indices.push(index as u32);
                    }
                }
            }
        } else if path.ends_with(".gltf") || path.ends_with(".glb") {
            let (gltf, buffers, _) = gltf::import(path)?;
            for mesh in gltf.meshes() {
                for primitive in mesh.primitives() {
                    let r = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                    if let Some(iter) = r.read_indices() {
                        for v in iter.into_u32() {
                            indices.push(v);
                        }
                    }
                    let mut positions = Vec::new();
                    if let Some(iter) = r.read_positions() {
                        for v in iter {
                            positions.push(v);
                        }
                    }
                    let mut uvs = Vec::new();
                    if let Some(gltf::mesh::util::ReadTexCoords::F32(
                        gltf::accessor::Iter::Standard(iter),
                    )) = r.read_tex_coords(0)
                    {
                        for v in iter {
                            uvs.push(v);
                        }
                    }
                    let mut normals = Vec::new();
                    if let Some(iter) = r.read_normals() {
                        for v in iter {
                            normals.push(v);
                        }
                    }

                    let size = positions.len();
                    for idx in 0..size {
                        let pos = positions[idx];
                        let normal = normals[idx];
                        let uv = uvs[idx];
                        let vertex = Vertex {
                            pos: Point3::new(
                                (pos[0] * scale) as f64,
                                (pos[2] * scale) as f64,
                                (pos[1] * scale) as f64,
                            ),
                            color: Vector3::new(1.0, 1.0, 1.0),
                            normal: Vector3::new(
                                normal[0] as f64,
                                normal[2] as f64,
                                normal[1] as f64,
                            ),
                            tex_coord: Vector2::new(uv[0] as f64, (1.0 - uv[1]) as f64),
                        };
                        vertices.push(vertex);
                        bbox.append(&vertex.pos);
                    }
                }
            }
        }
        let num = indices.len() / 3;
        for idx in 0..num {
            triangles.add(Arc::new(Triangle::new(
                vertices[indices[idx * 3] as usize].clone(),
                vertices[indices[idx * 3 + 1] as usize].clone(),
                vertices[indices[idx * 3 + 2] as usize].clone(),
                Arc::clone(&mat),
            )));
        }
        let triangles = HittableList::new(Arc::new(BvhNode::new(&mut triangles)));

        indices.clear();
        vertices.clear();
        Ok(Self {
            bbox,
            triangles,
            mat,
        })
    }
}

impl Hit for Model {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut ray_t = ray_t.clone();
        /*if !self.bbox.hit(r, &mut ray_t) {
            return false;
        }*/

        self.triangles.hit(r, &ray_t, rec)
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }

    fn pdf_value(&self, origin: Point3<f64>, direction: Vector3<f64>) -> f64 {
        self.triangles.pdf_value(origin, direction)
    }

    fn random(&self, origin: Point3<f64>) -> Vector3<f64> {
        self.triangles.random(origin)
    }
}
