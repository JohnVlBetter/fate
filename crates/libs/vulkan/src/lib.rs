mod buffer;
mod context;
mod debug;
mod descriptor;
mod image;
mod msaa;
mod pipeline;
mod shader;
mod swapchain;
mod texture;
mod util;
mod vertex;

pub use self::{
    buffer::*, context::*, debug::*, descriptor::*, image::*, msaa::*, pipeline::*, shader::*,
    swapchain::*, texture::*, util::*, vertex::*,
};

pub use ash;
use ash::vk;
use std::sync::Arc;
pub use winit;

pub struct PreLoadedResource<R, T> {
    context: Arc<Context>,
    command_buffer: vk::CommandBuffer,
    resource: Option<R>,
    tmp_data: Option<T>,
}

impl<R, T> PreLoadedResource<R, T> {
    pub fn new(
        context: Arc<Context>,
        command_buffer: vk::CommandBuffer,
        resource: R,
        tmp_data: T,
    ) -> Self {
        Self {
            context,
            command_buffer,
            resource: Some(resource),
            tmp_data: Some(tmp_data),
        }
    }
}

impl<R, T> PreLoadedResource<R, T> {
    pub fn finish(&mut self) -> R {
        assert!(self.resource.is_some(), "资源加载完成！");

        self.execute_commands();
        self.free_command_buffer();
        self.tmp_data.take();

        self.resource.take().unwrap()
    }

    fn execute_commands(&self) {
        self.context
            .execute_one_time_commands(|primary_command_buffer| unsafe {
                let secondary_command_buffer = [self.command_buffer];
                self.context
                    .device()
                    .cmd_execute_commands(primary_command_buffer, &secondary_command_buffer);
            });
    }

    fn free_command_buffer(&self) {
        let secondary_command_buffer = [self.command_buffer];
        unsafe {
            self.context.device().free_command_buffers(
                self.context.general_command_pool(),
                &secondary_command_buffer,
            )
        }
    }
}
