use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::device::VkDevice;
use crate::frame_buffer::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct RenderPass {
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub unsafe fn new(
        instance: &Instance,
        device: &VkDevice,
        swapchain_format: vk::Format,
    ) -> Result<Self> {
        // Attachments

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(device.msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(get_depth_format(instance, device.physical_device)?)
            .samples(device.msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        // Subpasses

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment_ref = vk::AttachmentReference::builder()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];
        let resolve_attachments = &[color_resolve_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .depth_stencil_attachment(&depth_stencil_attachment_ref)
            .resolve_attachments(resolve_attachments);

        // Dependencies

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );

        // Create

        let attachments = &[
            color_attachment,
            depth_stencil_attachment,
            color_resolve_attachment,
        ];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);

        let render_pass = device.device.create_render_pass(&info, None)?;

        Ok(Self { render_pass })
    }

    pub unsafe fn destory(&mut self, device: &VkDevice) {
        device.device.destroy_render_pass(self.render_pass, None);
    }
}
