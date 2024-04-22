use super::{context::*, util::*};
use ash::vk;
use std::{
    ffi::c_void,
    marker::{Send, Sync},
    mem::size_of,
    sync::Arc,
};

struct MemoryMapPointer(*mut c_void);
unsafe impl Send for MemoryMapPointer {}
unsafe impl Sync for MemoryMapPointer {}

pub struct Buffer {
    context: Arc<Context>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    mapped_pointer: Option<MemoryMapPointer>,
}

impl Buffer {
    fn new(
        context: Arc<Context>,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        size: vk::DeviceSize,
    ) -> Self {
        Self {
            context,
            buffer,
            memory,
            size,
            mapped_pointer: None,
        }
    }

    pub fn create(
        context: Arc<Context>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        mem_properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let device = context.device();
        let buffer = {
            let buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            unsafe {
                device
                    .create_buffer(&buffer_info, None)
                    .expect("Failed to create buffer")
            }
        };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory = {
            let mem_type = find_memory_type(
                mem_requirements,
                context.get_mem_properties(),
                mem_properties,
            );

            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                .memory_type_index(mem_type);
            unsafe {
                device
                    .allocate_memory(&alloc_info, None)
                    .expect("申请内存失败！")
            }
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("绑定buffer内存失败！")
        };

        Buffer::new(context, buffer, memory, size)
    }
}

impl Buffer {
    pub fn cmd_copy(&self, command_buffer: vk::CommandBuffer, src: &Buffer, size: vk::DeviceSize) {
        let region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        };
        let regions = [region];

        unsafe {
            self.context
                .device()
                .cmd_copy_buffer(command_buffer, src.buffer, self.buffer, &regions)
        };
    }

    pub fn map_memory(&mut self) -> *mut c_void {
        if let Some(ptr) = &self.mapped_pointer {
            ptr.0
        } else {
            unsafe {
                let ptr = self
                    .context
                    .device()
                    .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                    .expect("map memory失败！");
                self.mapped_pointer = Some(MemoryMapPointer(ptr));
                ptr
            }
        }
    }

    pub fn unmap_memory(&mut self) {
        if self.mapped_pointer.take().is_some() {
            unsafe {
                self.context.device().unmap_memory(self.memory);
            }
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.unmap_memory();
            self.context.device().destroy_buffer(self.buffer, None);
            self.context.device().free_memory(self.memory, None);
        }
    }
}

pub fn create_device_local_buffer_with_data<A, T: Copy>(
    context: &Arc<Context>,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> Buffer {
    let (buffer, _) = context.execute_one_time_commands(|command_buffer| {
        cmd_create_device_local_buffer_with_data::<A, _>(context, command_buffer, usage, data)
    });
    buffer
}

pub fn cmd_create_device_local_buffer_with_data<A, T: Copy>(
    context: &Arc<Context>,
    command_buffer: vk::CommandBuffer,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> (Buffer, Buffer) {
    let size = (data.len() * size_of::<T>()) as vk::DeviceSize;
    let staging_buffer =
        create_host_visible_buffer(context, vk::BufferUsageFlags::TRANSFER_SRC, data);
    let buffer = Buffer::create(
        Arc::clone(context),
        size,
        vk::BufferUsageFlags::TRANSFER_DST | usage,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    buffer.cmd_copy(command_buffer, &staging_buffer, staging_buffer.size);

    (buffer, staging_buffer)
}

pub fn create_host_visible_buffer<T: Copy>(
    context: &Arc<Context>,
    usage: vk::BufferUsageFlags,
    data: &[T],
) -> Buffer {
    let size = (data.len() * size_of::<T>()) as vk::DeviceSize;
    let mut buffer = Buffer::create(
        Arc::clone(context),
        size,
        usage,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        let data_ptr = buffer.map_memory();
        mem_copy(data_ptr, data);
    };

    buffer
}
