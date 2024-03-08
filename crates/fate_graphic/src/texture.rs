use crate::{buffer::*, tools::*};
use std::{collections::HashSet, ptr::copy_nonoverlapping as memcpy};

use image::{codecs::hdr::HdrDecoder, Rgb};
use std::{fs::File, io::BufReader, path::Path};

use anyhow::{anyhow, Result};
use gltf::{
    image::{Data, Format},
    iter::Materials,
};
use vulkanalia::prelude::v1_0::*;

use crate::device::VkDevice;

#[derive(Copy, Clone, Debug, Default)]
pub struct Texture {
    // Texture
    pub mip_levels: u32,
    pub is_srgb: bool,
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
}

impl Texture {
    pub unsafe fn new(
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        is_srgb: bool,
        instance: &Instance,
        device: &VkDevice,
    ) -> Result<Self> {
        let size = pixels.len() as u64;
        let mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            &device.device,
            device.physical_device,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = device.device.map_memory(
            staging_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(pixels.as_ptr(), memory.cast(), pixels.len());

        device.device.unmap_memory(staging_buffer_memory);

        let format = is_srgb
            .then_some(vk::Format::R8G8B8A8_SRGB)
            .unwrap_or(vk::Format::R8G8B8A8_UNORM);

        let (texture_image, texture_image_memory) = create_image(
            instance,
            &device.device,
            device.physical_device,
            width,
            height,
            mip_levels,
            vk::SampleCountFlags::_1,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let texture_image = texture_image;
        let texture_image_memory = texture_image_memory;

        transition_image_layout(
            &device.device,
            device.graphics_queue,
            device.command_pool,
            texture_image,
            format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
        )?;

        copy_buffer_to_image(
            &device.device,
            device.graphics_queue,
            device.command_pool,
            staging_buffer,
            texture_image,
            width,
            height,
        )?;

        // Cleanup

        device.device.destroy_buffer(staging_buffer, None);
        device.device.free_memory(staging_buffer_memory, None);

        // Mipmaps
        generate_mipmaps(
            instance,
            &device.device,
            device.physical_device,
            device.graphics_queue,
            device.command_pool,
            texture_image,
            format,
            width,
            height,
            mip_levels,
        )?;

        let texture_image_view = create_image_view(
            &device.device,
            texture_image,
            format,
            vk::ImageAspectFlags::COLOR,
            mip_levels,
        )?;

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
            .max_lod(mip_levels as f32)
            .mip_lod_bias(0.0);

        let texture_sampler = device.device.create_sampler(&info, None)?;

        Ok(Self {
            mip_levels,
            is_srgb,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
        })
    }

    pub unsafe fn destory(&mut self, device: &VkDevice) -> () {
        device.device.destroy_sampler(self.texture_sampler, None);
        device
            .device
            .destroy_image_view(self.texture_image_view, None);
        device.device.free_memory(self.texture_image_memory, None);
        device.device.destroy_image(self.texture_image, None);
    }
}

pub unsafe fn create_textures_from_gltf(
    materials: Materials,
    images: &[Data],
    instance: &Instance,
    device: &VkDevice,
) -> Vec<Texture> {
    let srgb_indices = {
        let mut indices = HashSet::new();

        for m in materials {
            if let Some(t) = m.pbr_metallic_roughness().base_color_texture() {
                indices.insert(t.texture().source().index());
            }

            if let Some(pbr_specular_glossiness) = m.pbr_specular_glossiness() {
                if let Some(t) = pbr_specular_glossiness.diffuse_texture() {
                    indices.insert(t.texture().source().index());
                }

                if let Some(t) = pbr_specular_glossiness.specular_glossiness_texture() {
                    indices.insert(t.texture().source().index());
                }
            }

            if let Some(t) = m.emissive_texture() {
                indices.insert(t.texture().source().index());
            }
        }

        indices
    };

    let textures: Vec<Texture> = images
        .iter()
        .enumerate()
        .map(|(index, image)| {
            let pixels = get_rgba_pixels(image);
            let is_srgb = srgb_indices.contains(&index);
            Texture::new(pixels, image.width, image.height, is_srgb, instance, device).unwrap()
        })
        .collect();

    textures
}

fn get_rgba_pixels(image: &Data) -> Vec<u8> {
    let mut buffer = Vec::new();
    let size = image.width * image.height;
    for index in 0..size {
        let rgba = next_rgba(&image.pixels, image.format, index as usize);
        buffer.extend_from_slice(&rgba);
    }
    buffer
}

fn next_rgba(pixels: &[u8], format: Format, index: usize) -> [u8; 4] {
    use Format::*;
    match format {
        R8 => [pixels[index], pixels[index], pixels[index], std::u8::MAX],
        R8G8 => [
            pixels[index * 2],
            pixels[index * 2],
            pixels[index * 2],
            pixels[index * 2 + 1],
        ],
        R8G8B8 => [
            pixels[index * 3],
            pixels[index * 3 + 1],
            pixels[index * 3 + 2],
            std::u8::MAX,
        ],
        R8G8B8A8 => [
            pixels[index * 4],
            pixels[index * 4 + 1],
            pixels[index * 4 + 2],
            pixels[index * 4 + 3],
        ],
        R16 | R16G16 | R16G16B16 | R16G16B16A16 | R32G32B32FLOAT | R32G32B32A32FLOAT => {
            panic!("不支持此纹理格式！")
        }
    }
}

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

unsafe fn generate_mipmaps(
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

pub fn load_hdr_image<P: AsRef<Path>>(path: P) -> (u32, u32, Vec<f32>) {
    let decoder = HdrDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap();
    let (width, height) = (decoder.metadata().width, decoder.metadata().height);
    let rgb = decoder.read_image_hdr().unwrap();
    let mut data = Vec::with_capacity(rgb.len() * 4);
    for Rgb(p) in rgb.iter() {
        data.extend_from_slice(p);
        data.push(0.0);
    }
    (width, height, data)
}