use crate::{
    buffer::{create_buffer, Buffer},
    device::VkDevice,
    model,
};
use anyhow::Result;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::prelude::v1_0::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub view: model::Mat4,
    pub proj: model::Mat4,
    pub color: model::Vec4,
}

#[derive(Clone, Debug, Default)]
pub struct UniformBuffer {
    pub buffer: Buffer,
}

impl UniformBuffer {
    pub unsafe fn new(instance: &Instance, device: &VkDevice) -> Result<Self> {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            &device.device,
            device.physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        Ok(UniformBuffer {
            buffer: Buffer {
                buffer: uniform_buffer,
                buffer_memory: uniform_buffer_memory,
            },
        })
    }

    pub unsafe fn update(&self, ubo: &UniformBufferObject, device: &VkDevice) -> Result<()> {
        let memory = device.device.map_memory(
            self.buffer.buffer_memory,
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(ubo, memory.cast(), 1);

        device.device.unmap_memory(self.buffer.buffer_memory);
        Ok(())
    }

    pub unsafe fn destory(&mut self, device: &VkDevice) {
        device.device.free_memory(self.buffer.buffer_memory, None);
        device.device.destroy_buffer(self.buffer.buffer, None);
    }
}
