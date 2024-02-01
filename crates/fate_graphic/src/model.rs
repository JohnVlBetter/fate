use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::Path;

use anyhow::Result;
use cgmath::{vec2, vec3};
use gltf::image::Source;
use gltf::Gltf;
use image::ImageFormat::{JPEG, PNG};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;

use crate::buffer::{copy_buffer, create_buffer, Buffer};
use crate::device::VkDevice;
use crate::transform::Transform;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: Vec3,
    color: Vec3,
    normal: Vec3,
    tex_coord: Vec2,
}

impl Vertex {
    pub fn new(pos: Vec3, color: Vec3, normal: Vec3, tex_coord: Vec2) -> Self {
        Self {
            pos,
            color,
            normal,
            tex_coord,
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32)
            .build();
        let normal = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32)
            .build();
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(3)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>() + size_of::<Vec3>()) as u32)
            .build();
        [pos, color, normal, tex_coord]
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.normal == other.normal
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    //现在没接ecs transform先放这
    pub transform: Transform,
}

impl Model {
    pub unsafe fn new(path: &str, instance: &Instance, device: &VkDevice) -> Result<Self> {
        let mut unique_vertices = HashMap::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

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
                        pos: vec3(
                            model.mesh.positions[pos_offset],
                            model.mesh.positions[pos_offset + 1],
                            model.mesh.positions[pos_offset + 2],
                        ),
                        color: vec3(1.0, 1.0, 1.0),
                        normal: vec3(
                            model.mesh.normals[normal_index * 3],
                            model.mesh.normals[normal_index * 3 + 1],
                            model.mesh.normals[normal_index * 3 + 2],
                        ),
                        tex_coord: vec2(
                            model.mesh.texcoords[tex_coord_offset],
                            1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                        ),
                    };

                    if let Some(index) = unique_vertices.get(&vertex) {
                        indices.push(*index as u32);
                    } else {
                        let index = vertices.len();
                        unique_vertices.insert(vertex, index);
                        vertices.push(vertex);
                        indices.push(index as u32);
                    }
                }
            }
        } else if path.ends_with(".gltf") || path.ends_with(".glb") {
            let (gltf, buffers, images) = gltf::import(path)?;
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
                            pos: vec3(pos[0], pos[2], pos[1]),
                            color: vec3(1.0, 1.0, 1.0),
                            normal: vec3(normal[0], normal[2], normal[1]),
                            tex_coord: vec2(uv[0], 1.0 - uv[1]),
                        };
                        vertices.push(vertex);
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
                            "image/jpeg" => image::load_from_memory_with_format(data, JPEG),
                            "image/png" => image::load_from_memory_with_format(data, PNG),
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
                                "image/jpeg" => image::load_from_memory_with_format(&data, JPEG),
                                "image/png" => image::load_from_memory_with_format(&data, PNG),
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
                                "image/jpeg" => image::load(reader, JPEG),
                                "image/png" => image::load(reader, PNG),
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
                            let path = Path::new(path)
                                .parent()
                                .unwrap_or_else(|| Path::new("./"))
                                .join(uri);
                            image::open(path)
                        }
                    }
                };
            }
        }

        let vertex_buffer = create_vertex_buffer(instance, device, &vertices)?;
        let index_buffer = create_index_buffer(instance, device, &indices)?;

        let transform = Transform::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        )?;

        Ok(Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
            transform,
        })
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        self.vertices.clear();
        self.indices.clear();
        device.destory_buffer(&self.index_buffer);
        device.destory_buffer(&self.vertex_buffer);
    }
}

unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &VkDevice,
    vertices: &Vec<Vertex>,
) -> Result<Buffer> {
    let size = (size_of::<Vertex>() * vertices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        &device.device,
        device.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory =
        device
            .device
            .map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(vertices.as_ptr(), memory.cast(), vertices.len());

    device.device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        instance,
        &device.device,
        device.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    let buffer: Buffer = Buffer {
        buffer: vertex_buffer,
        buffer_memory: vertex_buffer_memory,
    };

    copy_buffer(
        &device.device,
        device.graphics_queue,
        device.command_pool,
        staging_buffer,
        buffer.buffer,
        size,
    )?;

    device.device.destroy_buffer(staging_buffer, None);
    device.device.free_memory(staging_buffer_memory, None);

    Ok(buffer)
}

unsafe fn create_index_buffer(
    instance: &Instance,
    device: &VkDevice,
    indices: &Vec<u32>,
) -> Result<Buffer> {
    let size = (size_of::<u32>() * indices.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        &device.device,
        device.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory =
        device
            .device
            .map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(indices.as_ptr(), memory.cast(), indices.len());

    device.device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        instance,
        &device.device,
        device.physical_device,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    let buffer: Buffer = Buffer {
        buffer: index_buffer,
        buffer_memory: index_buffer_memory,
    };

    copy_buffer(
        &device.device,
        device.graphics_queue,
        device.command_pool,
        staging_buffer,
        buffer.buffer,
        size,
    )?;

    device.device.destroy_buffer(staging_buffer, None);
    device.device.free_memory(staging_buffer_memory, None);

    Ok(buffer)
}
