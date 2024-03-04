use crate::{mikktspace::generate_tangents, texture::Texture};
use anyhow::Result;
use cgmath::{vec2, vec3, vec4};
use gltf::{
    buffer::Buffer as GltfBuffer,
    mesh::{Reader, Semantic},
};
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
    pub pos: Vec3,
    pub color: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
    pub tangent: Vec4,
}

impl Vertex {
    pub fn new(pos: Vec3, color: Vec3, normal: Vec3, tex_coord: Vec2, tangent: Vec4) -> Self {
        Self {
            pos,
            color,
            normal,
            tex_coord,
            tangent,
        }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 5] {
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
        [pos, color, normal, tex_coord, tangent]
    }
}

impl PartialEq for Vertex {
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
}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    //现在没接ecs transform先放这
    pub transform: Transform,

    pub albedo_texture: Texture,
    pub normal_texture: Texture,
    pub material_texture: Texture,
    pub emissive_texture: Texture,
    pub ao_texture: Texture,
}

impl Model {
    pub unsafe fn new(path: &str, instance: &Instance, device: &VkDevice) -> Result<Self> {
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        let mut textures: Vec<Texture> = Vec::new();
        let mut material_image_index: Vec<i32> = vec![-1; 5];
        let (document, buffers, images) = gltf::import(path)?;
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(_accessor) = primitive.get(&Semantic::Positions) {
                    let positions = read_positions(&reader);
                    let normals = read_normals(&reader);
                    let tex_coords_0 = read_tex_coords(&reader, 0);
                    let tex_coords_1 = read_tex_coords(&reader, 1);
                    let tangents = read_tangents(&reader);
                    let colors = read_colors(&reader);

                    vertices = positions
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
                                pos: vec3(position[0], position[2], position[1]),
                                color: vec3(colors[0], colors[1], colors[2]),
                                normal: vec3(normal[0], normal[2], normal[1]),
                                tex_coord: vec2(tex_coords_0[0], tex_coords_0[1]),
                                tangent: vec4(tangent[0], tangent[1], tangent[2], tangent[3]),
                            }
                        })
                        .collect::<Vec<_>>();

                    indices = read_indices(&reader).unwrap();

                    if !positions.is_empty()
                        && !normals.is_empty()
                        && !tex_coords_0.is_empty()
                        && tangents.is_empty()
                    {
                        generate_tangents(read_indices(&reader).as_deref(), &mut vertices);
                    }
                }
            }
        }

        images.iter().enumerate().for_each(|(_index, image)| {
            let mut pixels = Vec::new();
            let size = image.width * image.height;
            for index in 0..size {
                let rgba = [
                    image.pixels[index as usize * 3],
                    image.pixels[index as usize * 3 + 1],
                    image.pixels[index as usize * 3 + 2],
                    255,
                ];
                pixels.extend_from_slice(&rgba);
            }
            let new_texture =
                Texture::new(pixels, image.width, image.height, instance, device).unwrap();
            textures.push(new_texture);
        });

        for material in document.materials() {
            //albedo
            let color_texture_idx = match material.pbr_metallic_roughness().base_color_texture() {
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
            albedo_texture: textures[material_image_index[0] as usize],
            normal_texture: textures[material_image_index[1] as usize],
            material_texture: textures[material_image_index[2] as usize],
            emissive_texture: textures[material_image_index[4] as usize],
            ao_texture: textures[material_image_index[3] as usize],
        })
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        self.vertices.clear();
        self.indices.clear();
        self.albedo_texture.destory(&device);
        self.normal_texture.destory(&device);
        self.material_texture.destory(&device);
        self.ao_texture.destory(&device);
        self.emissive_texture.destory(&device);
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
