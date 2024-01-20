use crate::{buffer::*, tools::*};
use std::fs::File;
use std::ptr::copy_nonoverlapping as memcpy;

use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Texture {
    // Texture
    pub mip_levels: u32,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
}

impl Texture {
    //================================================
    // Texture
    //================================================

    pub unsafe fn create_texture_image(
        &mut self,
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
    ) -> Result<()> {
        // Load

        let image = File::open("res/model/viking_room/viking_room.png")?;

        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;

        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();
        self.mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;

        if width != 1024 || height != 1024 || reader.info().color_type != png::ColorType::Rgba {
            panic!("Invalid texture image (use https://kylemayes.github.io/vulkanalia/images/viking_room.png).");
        }

        // Create (staging)

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        // Copy (staging)

        let memory =
            device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

        memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

        device.unmap_memory(staging_buffer_memory);

        // Create (image)

        let (texture_image, texture_image_memory) = create_image(
            instance,
            device,
            physical_device,
            width,
            height,
            self.mip_levels,
            vk::SampleCountFlags::_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        self.texture_image = texture_image;
        self.texture_image_memory = texture_image_memory;

        // Transition + Copy (image)

        transition_image_layout(
            device,
            graphics_queue,
            command_pool,
            self.texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            self.mip_levels,
        )?;

        copy_buffer_to_image(
            device,
            graphics_queue,
            command_pool,
            staging_buffer,
            self.texture_image,
            width,
            height,
        )?;

        // Cleanup

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        // Mipmaps

        Self::generate_mipmaps(
            instance,
            device,
            physical_device,
            graphics_queue,
            command_pool,
            self.texture_image,
            vk::Format::R8G8B8A8_SRGB,
            width,
            height,
            self.mip_levels,
        )?;

        Ok(())
    }

    pub unsafe fn generate_mipmaps(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        image: vk::Image,
        format: vk::Format,
        width: u32,
        height: u32,
        mip_levels: u32,
    ) -> Result<()> {
        // Support

        if !instance
            .get_physical_device_format_properties(physical_device, format)
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            return Err(anyhow!(
                "Texture image format does not support linear blitting!"
            ));
        }

        // Mipmaps

        let command_buffer = begin_single_time_commands(device, command_pool)?;

        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .level_count(1);

        let mut barrier = vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(subresource);

        let mut mip_width = width;
        let mut mip_height = height;

        for i in 1..mip_levels {
            barrier.subresource_range.base_mip_level = i - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );

            let src_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i - 1)
                .base_array_layer(0)
                .layer_count(1);

            let dst_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(i)
                .base_array_layer(0)
                .layer_count(1);

            let blit = vk::ImageBlit::builder()
                .src_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width as i32,
                        y: mip_height as i32,
                        z: 1,
                    },
                ])
                .src_subresource(src_subresource)
                .dst_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                        y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                        z: 1,
                    },
                ])
                .dst_subresource(dst_subresource);

            device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::LINEAR,
            );

            barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );

            if mip_width > 1 {
                mip_width /= 2;
            }

            if mip_height > 1 {
                mip_height /= 2;
            }
        }

        barrier.subresource_range.base_mip_level = mip_levels - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );

        end_single_time_commands(device, graphics_queue, command_pool, command_buffer)?;

        Ok(())
    }

    pub unsafe fn create_texture_image_view(&mut self, device: &Device) -> Result<()> {
        self.texture_image_view = create_image_view(
            device,
            self.texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            self.mip_levels,
        )?;

        Ok(())
    }

    pub unsafe fn create_texture_sampler(&mut self, device: &Device) -> Result<()> {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .min_lod(0.0)
            .max_lod(self.mip_levels as f32)
            .mip_lod_bias(0.0);

        self.texture_sampler = device.create_sampler(&info, None)?;

        Ok(())
    }
}

//================================================
// Shared (Images)
//================================================
pub unsafe fn create_image(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    width: u32,
    height: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    // Image
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples);
    let image = device.create_image(&info, None)?;
    // Memory
    let requirements = device.get_image_memory_requirements(image);
    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            physical_device,
            properties,
            requirements,
        )?);
    let image_memory = device.allocate_memory(&info, None)?;
    device.bind_image_memory(image, image_memory, 0)?;
    Ok((image, image_memory))
}
pub unsafe fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspects: vk::ImageAspectFlags,
    mip_levels: u32,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);
    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);
    Ok(device.create_image_view(&info, None)?)
}
pub unsafe fn transition_image_layout(
    device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    image: vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32,
) -> Result<()> {
    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => return Err(anyhow!("Unsupported image layout transition!")),
        };
    let command_buffer = begin_single_time_commands(device, command_pool)?;
    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);
    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);
    device.cmd_pipeline_barrier(
        command_buffer,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );
    end_single_time_commands(device, graphics_queue, command_pool, command_buffer)?;
    Ok(())
}
unsafe fn copy_buffer_to_image(
    device: &Device,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(device, command_pool)?;
    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);
    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });
    device.cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
    );
    end_single_time_commands(device, graphics_queue, command_pool, command_buffer)?;
    Ok(())
}
