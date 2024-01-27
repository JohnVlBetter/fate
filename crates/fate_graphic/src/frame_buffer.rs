use std::rc::Rc;

use crate::{device::VkDevice, swapchain::Swapchain, texture::*};
use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;

#[derive(Clone, Debug, Default)]
pub struct FrameBuffer {
    pub color_attachment: Rc<ColorAttachment>,
    pub depth_attachment: Rc<DepthAttachment>,
    pub frame_buffer: vk::Framebuffer,
}

impl FrameBuffer {
    pub unsafe fn new(
        device: &Device,
        swapchain: &Swapchain,
        color_attachment: &Rc<ColorAttachment>,
        depth_attachment: &Rc<DepthAttachment>,
        render_pass: vk::RenderPass,
        idx: usize,
    ) -> Result<Self> {
        let attachments = &[
            color_attachment.color_image_view,
            depth_attachment.depth_image_view,
            swapchain.swapchain_image_views[idx],
        ];
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(swapchain.swapchain_extent.width)
            .height(swapchain.swapchain_extent.height)
            .layers(1);

        let frame_buffer = device.create_framebuffer(&create_info, None)?;

        Ok(Self {
            color_attachment: Rc::clone(color_attachment),
            depth_attachment: Rc::clone(depth_attachment),
            frame_buffer,
        })
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub struct ColorAttachment {
    pub color_image: vk::Image,
    pub color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,
}

impl ColorAttachment {
    pub unsafe fn new(
        instance: &Instance,
        device: &VkDevice,
        swapchain: &Swapchain,
    ) -> Result<Self> {
        let (color_image, color_image_memory) = create_image(
            instance,
            &device.device,
            device.physical_device,
            swapchain.swapchain_extent.width,
            swapchain.swapchain_extent.height,
            1,
            device.msaa_samples,
            swapchain.swapchain_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Image View

        let color_image_view = create_image_view(
            &device.device,
            color_image,
            swapchain.swapchain_format,
            vk::ImageAspectFlags::COLOR,
            1,
        )?;

        Ok(Self {
            color_image,
            color_image_memory,
            color_image_view,
        })
    }

    pub unsafe fn destory(&mut self, device: &VkDevice) {
        device
            .device
            .destroy_image_view(self.color_image_view, None);
        device.device.free_memory(self.color_image_memory, None);
        device.device.destroy_image(self.color_image, None);
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub struct DepthAttachment {
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
}

impl DepthAttachment {
    pub unsafe fn new(
        instance: &Instance,
        device: &VkDevice,
        swapchain: &Swapchain,
    ) -> Result<Self> {
        let format = get_depth_format(instance, device.physical_device)?;

        let (depth_image, depth_image_memory) = create_image(
            instance,
            &device.device,
            device.physical_device,
            swapchain.swapchain_extent.width,
            swapchain.swapchain_extent.height,
            1,
            device.msaa_samples,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view = create_image_view(
            &device.device,
            depth_image,
            format,
            vk::ImageAspectFlags::DEPTH,
            1,
        )?;

        Ok(Self {
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
    }

    pub unsafe fn destory(&mut self, device: &VkDevice) {
        device
            .device
            .destroy_image_view(self.depth_image_view, None);
        device.device.free_memory(self.depth_image_memory, None);
        device.device.destroy_image(self.depth_image, None);
    }
}

pub unsafe fn get_depth_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        physical_device,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

unsafe fn get_supported_format(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = instance.get_physical_device_format_properties(physical_device, *f);
            match tiling {
                vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
        .ok_or_else(|| anyhow!("Failed to find supported format!"))
}
