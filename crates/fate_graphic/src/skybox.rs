use anyhow::Result;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;

use crate::buffer::{copy_buffer, create_buffer, Buffer};
use crate::device::VkDevice;
use crate::vertex::Vertex;

pub type Vec2 = cgmath::Vector2<f32>;
pub type Vec3 = cgmath::Vector3<f32>;
pub type Vec4 = cgmath::Vector4<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct SkyboxVertex {
    position: Vec3,
}

impl SkyboxVertex {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}

impl Vertex for SkyboxVertex {
    fn binding_description() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<SkyboxVertex>() as u32)
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
        vec![pos]
    }
}

pub struct SkyboxModel {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl SkyboxModel {
    pub unsafe fn new(instance: &Instance, device: &VkDevice) -> Self {
        let indices: Vec<u32> = vec![
            0, 1, 2, 2, 3, 0, 1, 5, 6, 6, 2, 1, 5, 4, 7, 7, 6, 5, 4, 0, 3, 3, 7, 4, 3, 2, 6, 6, 7,
            3, 4, 5, 1, 1, 0, 4,
        ];
        let vertices: Vec<SkyboxVertex> = vec![
            SkyboxVertex::new(Vec3::new(-0.5, -0.5, -0.5)),
            SkyboxVertex::new(Vec3::new(0.5, -0.5, -0.5)),
            SkyboxVertex::new(Vec3::new(0.5, 0.5, -0.5)),
            SkyboxVertex::new(Vec3::new(-0.5, 0.5, -0.5)),
            SkyboxVertex::new(Vec3::new(-0.5, -0.5, 0.5)),
            SkyboxVertex::new(Vec3::new(0.5, -0.5, 0.5)),
            SkyboxVertex::new(Vec3::new(0.5, 0.5, 0.5)),
            SkyboxVertex::new(Vec3::new(-0.5, 0.5, 0.5)),
        ];
        let vertex_buffer = create_vertex_buffer(instance, device, &vertices).unwrap();
        let index_buffer = create_index_buffer(instance, device, &indices).unwrap();

        SkyboxModel {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn vertices_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn indices_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        device.destory_buffer(&self.index_buffer);
        device.destory_buffer(&self.vertex_buffer);
    }
}

unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &VkDevice,
    vertices: &Vec<SkyboxVertex>,
) -> Result<Buffer> {
    let size = (size_of::<SkyboxVertex>() * vertices.len()) as u64;

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
