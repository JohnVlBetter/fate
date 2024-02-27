use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use cgmath::{Point3, Vector2, Vector3};
use gltf::image::Source;
use gltf::json::extensions::material;
use image::GenericImageView;
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
    pub images: Vec<Image>,
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

                let (width, height) = (dyn_img.width(), dyn_img.height());
                let image_data = dyn_img.to_rgb8().into_vec();
                let new_image =
                    Image::new_with_data(width as usize, height as usize, image_data, 3);
                model_images.push(new_image);
            }
            for material in gltf.materials() {
                let cur_tex = material
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .unwrap()
                    .texture();
                material_image_index[0] = cur_tex.index() as i32;
            }
        }
        let material: Arc<dyn Scatter> = Arc::new(PBR::new(
            Arc::new(ImageTexture::new_with_image(
                model_images[material_image_index[0] as usize].clone(),
            )),
            Arc::new(ImageTexture::new_with_image(
                model_images[material_image_index[0] as usize].clone(),
            )),
        ));

        let num = indices.len() / 3;
        for idx in 0..num {
            triangles.add(Arc::new(Triangle::new(
                vertices[indices[idx * 3] as usize].clone(),
                vertices[indices[idx * 3 + 1] as usize].clone(),
                vertices[indices[idx * 3 + 2] as usize].clone(),
                Arc::clone(&material),
            )));
        }
        let triangles = HittableList::new(Arc::new(BvhNode::new(&mut triangles)));

        indices.clear();
        vertices.clear();
        Ok(Self {
            bbox,
            triangles,
            material,
            transform,
            images: model_images,
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
