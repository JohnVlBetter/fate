use crate::{material::Material, mikktspace::generate_tangents, vertex::Vertex};
use anyhow::Result;
use cgmath::{vec2, vec3, vec4};
use gltf::{
    buffer::{Buffer as GltfBuffer, Data},
    mesh::{Reader, Semantic},
    Document,
};
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;

use crate::buffer::{copy_buffer, create_buffer, Buffer};
use crate::device::VkDevice;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
    pub tangent: Vec4,
}

impl ModelVertex {
    pub fn new(pos: Vec3, color: Vec3, normal: Vec3, tex_coord: Vec2, tangent: Vec4) -> Self {
        Self {
            pos,
            color,
            normal,
            tex_coord,
            tangent,
        }
    }
}

impl Vertex for ModelVertex {
    fn binding_description() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<ModelVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
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
        let tangent = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(4)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(
                (size_of::<Vec3>() + size_of::<Vec3>() + size_of::<Vec3>() + size_of::<Vec2>())
                    as u32,
            )
            .build();
        vec![pos, color, normal, tex_coord, tangent]
    }
}

/*impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.normal == other.normal
            && self.tex_coord == other.tex_coord
            && self.tangent == other.tangent
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
        self.tangent[0].to_bits().hash(state);
        self.tangent[1].to_bits().hash(state);
        self.tangent[2].to_bits().hash(state);
        self.tangent[3].to_bits().hash(state);
    }
}*/

#[derive(Clone, Debug)]
pub struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}

#[derive(Clone, Debug)]
pub struct Primitive {
    index: usize,
    //pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    material: Material,
}

impl Primitive {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn vertices_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn indices_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn material(&self) -> Material {
        self.material
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        //self.vertices.clear();
        self.indices.clear();
        device.destory_buffer(&self.index_buffer);
        device.destory_buffer(&self.vertex_buffer);
    }
}

pub unsafe fn create_meshes_from_gltf(
    document: &Document,
    buffers: &[Data],
    instance: &Instance,
    device: &VkDevice,
) -> Vec<Mesh> {
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

                        ModelVertex {
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

                let vertex_buffer = create_vertex_buffer(instance, device, &vertices).unwrap();
                let index_buffer = create_index_buffer(instance, device, &indices).unwrap();

                let material = primitive.material().into();

                primitives.push(Primitive {
                    index: 1,
                    indices,
                    vertex_buffer,
                    index_buffer,
                    material,
                })
            }
        }
        meshes.push(Mesh { primitives })
    }
    meshes
}

unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &VkDevice,
    vertices: &Vec<ModelVertex>,
) -> Result<Buffer> {
    let size = (size_of::<ModelVertex>() * vertices.len()) as u64;

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
