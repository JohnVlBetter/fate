use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use cgmath::{Point3, Vector2, Vector3};
use gltf::image::Source;
use image::ImageFormat::{Jpeg, Png};
use std::collections::HashMap;

use crate::aabb::{Aabb, EMPTY};
use crate::bvh::BvhNode;
use crate::hit::{Hit, HitRecord};
use crate::hittable_list::HittableList;
use crate::image::Image;
use crate::interval::Interval;
use crate::material::{Scatter, PBR};
use crate::ray::Ray;
use crate::texture::ImageTexture;
use crate::transform::Transform;
use crate::triangle::{Triangle, Vertex};

pub struct Model {
    pub bbox: Aabb,
    pub triangles: HittableList,
    pub material: Arc<dyn Scatter>,
    pub transform: Transform,
}

impl Model {
    pub fn new(path: &str, scale: f32, transform: Transform) -> Result<Self> {
        let mut unique_vertices = HashMap::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut bbox: Aabb = EMPTY;
        let mut triangles = HittableList::default();

        let mut model_images: Vec<Image> = Vec::new();
        let mut material_image_index: Vec<i32> = vec![-1; 5];
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
            let (gltf, buffers, _images) = gltf::import(path)?;
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
                    let mut tex_coord_set = 0;
                    while let Some(tex_coords) = r.read_tex_coords(tex_coord_set) {
                        if tex_coord_set > 1 {
                            println!("忽略槽位{},只支持两套uv", tex_coord_set);
                            tex_coord_set += 1;
                            continue;
                        }
                        for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                            match tex_coord_set {
                                0 => {
                                    uvs.push(Vector2::new(tex_coord[0] as f64, tex_coord[1] as f64))
                                }
                                //1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                                _ => unreachable!(),
                            }
                        }
                        tex_coord_set += 1;
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
                            tex_coord: Vector2::new(uv[0] as f64, (uv[1] - 1.0) as f64),
                        };
                        vertices.push(vertex);
                        bbox.append(&vertex.pos);
                    }
                }
            }
            for image in gltf.images() {
                let img = match image.source() {
                    Source::View { view, mime_type } => {
                        let parent_buffer_data = &buffers[view.buffer().index()].0;
                        let begin = view.offset();
                        let end = begin + view.length();
                        let data = &parent_buffer_data[begin..end];
                        match mime_type {
                            "image/jpeg" => image::load_from_memory_with_format(data, Jpeg),
                            "image/png" => image::load_from_memory_with_format(data, Png),
                            _ => panic!(
                                "{}",
                                format!(
                                    "unsupported image type (image: {}, mime_type: {})",
                                    image.index(),
                                    mime_type
                                )
                            ),
                        }
                    }
                    Source::Uri { uri, mime_type } => {
                        if uri.starts_with("data:") {
                            let encoded = uri.split(',').nth(1).unwrap();
                            let data = base64::decode(&encoded).unwrap();
                            let mime_type = if let Some(ty) = mime_type {
                                ty
                            } else {
                                uri.split(',')
                                    .nth(0)
                                    .unwrap()
                                    .split(':')
                                    .nth(1)
                                    .unwrap()
                                    .split(';')
                                    .nth(0)
                                    .unwrap()
                            };

                            match mime_type {
                                "image/jpeg" => image::load_from_memory_with_format(&data, Jpeg),
                                "image/png" => image::load_from_memory_with_format(&data, Png),
                                _ => panic!(
                                    "{}",
                                    format!(
                                        "unsupported image type (image: {}, mime_type: {})",
                                        image.index(),
                                        mime_type
                                    )
                                ),
                            }
                        } else if let Some(mime_type) = mime_type {
                            let path = Path::new(path)
                                .parent()
                                .unwrap_or_else(|| Path::new("./"))
                                .join(uri);
                            let file = fs::File::open(path).unwrap();
                            let reader = io::BufReader::new(file);
                            match mime_type {
                                "image/jpeg" => image::load(reader, Jpeg),
                                "image/png" => image::load(reader, Png),
                                _ => panic!(
                                    "{}",
                                    format!(
                                        "unsupported image type (image: {}, mime_type: {})",
                                        image.index(),
                                        mime_type
                                    )
                                ),
                            }
                        } else {
                            let image_path = Path::new(path)
                                .parent()
                                .unwrap_or_else(|| Path::new("./"))
                                .join(uri);
                            println!("Loading {}", image_path.to_str().unwrap());
                            image::open(image_path)
                        }
                    }
                };
                let dyn_img: image::DynamicImage = img.expect("Image loading failed.");
                let new_image = Image::new_with_dyn_img(dyn_img);
                model_images.push(new_image);
            }
            for material in gltf.materials() {
                //albedo
                let color_texture_idx = match material.pbr_metallic_roughness().base_color_texture()
                {
                    Some(color_texture) => color_texture.texture().index() as i32,
                    None => -1,
                };
                material_image_index[0] = color_texture_idx;

                //normal
                let normal_texture_idx = match material.normal_texture() {
                    Some(normal_texture) => normal_texture.texture().index() as i32,
                    None => -1,
                };
                material_image_index[1] = normal_texture_idx;

                //metallic_roughness
                let metallic_roughness_texture_idx = match material
                    .pbr_metallic_roughness()
                    .metallic_roughness_texture()
                {
                    Some(metallic_roughness_texture) => {
                        metallic_roughness_texture.texture().index() as i32
                    }
                    None => -1,
                };
                material_image_index[2] = metallic_roughness_texture_idx;

                //ao
                let occlusion_texture_idx = match material.occlusion_texture() {
                    Some(occlusion_texture) => occlusion_texture.texture().index() as i32,
                    None => -1,
                };
                material_image_index[3] = occlusion_texture_idx;

                //emissive
                let emissive_texture_idx = match material.emissive_texture() {
                    Some(emissive_texture) => emissive_texture.texture().index() as i32,
                    None => -1,
                };
                material_image_index[4] = emissive_texture_idx;
            }
        }
        let material: Arc<dyn Scatter> = Arc::new(PBR::new(
            Arc::new(ImageTexture::new_with_image(
                model_images[material_image_index[0] as usize].clone(),
            )),
            Arc::new(ImageTexture::new_with_image(
                model_images[material_image_index[2] as usize].clone(),
            )),
        ));
        let normal_image = Arc::new(model_images[material_image_index[1] as usize].clone());

        let num = indices.len() / 3;
        for idx in 0..num {
            triangles.add(Arc::new(Triangle::new(
                vertices[indices[idx * 3] as usize].clone(),
                vertices[indices[idx * 3 + 1] as usize].clone(),
                vertices[indices[idx * 3 + 2] as usize].clone(),
                Arc::clone(&material),
                Arc::clone(&normal_image),
            )));
        }
        let triangles = HittableList::new(Arc::new(BvhNode::new(&mut triangles)));

        //let metallic_roughness_image = model_images[material_image_index[2] as usize].clone();

        indices.clear();
        vertices.clear();
        Ok(Self {
            bbox,
            triangles,
            material,
            transform,
        })
    }
}

impl Hit for Model {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut ray_t = ray_t.clone();
        let r = self.transform.transform_ray(r);
        if !self.bbox.hit(&r, &mut ray_t) {
            return false;
        }

        let res = self.triangles.hit(&r, &ray_t, rec);
        self.transform.transform_rec(rec);
        res
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
