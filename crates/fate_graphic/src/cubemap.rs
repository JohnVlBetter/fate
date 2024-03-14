use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use std::mem::size_of;
use std::path::Path;
use std::time::Instant;
use vulkanalia::prelude::v1_0::*;

use crate::{
    descriptors::create_descriptors,
    device::VkDevice,
    pipeline::{create_pipeline, PipelineParameters},
    skybox::{SkyboxModel, SkyboxVertex},
    texture::{generate_mipmaps, load_hdr_image, transition_image_layout, Texture},
    tools::{begin_single_time_commands, end_single_time_commands},
    vertex::Vertex,
};

pub(crate) unsafe fn create_skybox_cubemap<P: AsRef<Path>>(
    instance: &Instance,
    device: &VkDevice,
    swapchain_format: vk::Format,
    path: P,
    size: u32,
) -> Texture {
    log::info!("Creating cubemap from equirectangular texture");
    let start = Instant::now();
    let (w, h, data) = load_hdr_image(path);
    let mip_levels = (size as f32).log2().floor() as u32 + 1;

    let cubemap_format = vk::Format::R16G16B16A16_SFLOAT;

    let texture = Texture::new_hdr(data, w, h, true, instance, device).unwrap();
    let cubemap =
        Texture::create_renderable_cubemap(instance, device, size, mip_levels, cubemap_format);

    let skybox_model = SkyboxModel::new(instance, device);

    let descriptors = create_descriptors(instance, &device.device, &texture);

    let (pipeline_layout, pipeline) = {
        let layout = {
            let layouts = [descriptors.layout()];
            let push_constant_range = [vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::VERTEX,
                offset: 0,
                size: size_of::<Matrix4<f32>>() as _,
            }];
            let layout_info = vk::PipelineLayoutCreateInfo::builder()
                .set_layouts(&layouts)
                .push_constant_ranges(&push_constant_range);

            device
                .device
                .create_pipeline_layout(&layout_info, None)
                .unwrap()
        };

        let pipeline = {
            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(size as _)
                .height(size as _)
                .min_depth(0.0)
                .max_depth(1.0)
                .build();

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(vk::Extent2D {
                    width: size,
                    height: size,
                });

            let viewports = &[viewport];
            let scissors = &[scissor];
            let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .viewports(viewports)
                .scissors(scissors);

            let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::FRONT)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0);

            create_env_pipeline::<SkyboxVertex>(
                device,
                EnvPipelineParameters {
                    viewport_info: &viewport_state,
                    rasterizer_info: &rasterizer_info,
                    dynamic_state_info: None,
                    layout,
                    format: cubemap_format,
                },
            )
        };

        (layout, pipeline)
    };

    let views = (0..6)
        .map(|i| {
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(cubemap.texture_image)
                .view_type(vk::ImageViewType::_2D)
                .format(cubemap_format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: i,
                    layer_count: 1,
                });

            device.device.create_image_view(&create_info, None).unwrap()
        })
        .collect::<Vec<_>>();

    let view_matrices = get_view_matrices();

    let proj = perspective(Deg(90.0), 1.0, 0.1, 10.0);

    // Render
    let command_buffer = begin_single_time_commands(&device.device, device.command_pool).unwrap();
    for face in 0..6 {
        /*let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(device.msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_resolve_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let color_attachments = &[color_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .resolve_attachments(resolve_attachments);*/
        let attachment_info = RenderingAttachmentInfo::builder()
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            })
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .image_view(views[face])
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let rendering_info = RenderingInfo::builder()
            .color_attachments(std::slice::from_ref(&attachment_info))
            .layer_count(1)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: size,
                    height: size,
                },
            });

        context
            .dynamic_rendering()
            .cmd_begin_rendering(buffer, &rendering_info);

        device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);

        device.cmd_bind_descriptor_sets(
            buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &descriptors.sets()[0..=0],
            &[],
        );

        let view = view_matrices[face];
        let view_proj = proj * view;
        let push = any_as_u8_slice(&view_proj);
        device.cmd_push_constants(
            buffer,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            push,
        );

        device.cmd_bind_vertex_buffers(buffer, 0, &[skybox_model.vertices_buffer().buffer], &[0]);
        device.cmd_bind_index_buffer(
            buffer,
            skybox_model.indices_buffer().buffer,
            0,
            vk::IndexType::UINT32,
        );

        // Draw skybox
        device.cmd_draw_indexed(buffer, 36, 1, 0, 0, 0);

        // End render pass
        context.dynamic_rendering().cmd_end_rendering(buffer);
    }
    end_single_time_commands(
        &device.device,
        device.graphics_queue,
        device.command_pool,
        command_buffer,
    )
    .unwrap();

    transition_image_layout(
        &device.device,
        device.graphics_queue,
        device.command_pool,
        cubemap.texture_image,
        cubemap_format,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels,
        6,
    )
    .unwrap();

    generate_mipmaps(
        instance,
        &device.device,
        device.physical_device,
        device.graphics_queue,
        device.command_pool,
        cubemap.texture_image,
        cubemap_format,
        size,
        size,
        mip_levels,
        6,
    )
    .unwrap();

    // Cleanup
    views
        .iter()
        .for_each(|v| device.device.destroy_image_view(*v, None));
    device.device.destroy_pipeline(pipeline, None);
    device.device.destroy_pipeline_layout(pipeline_layout, None);

    let time = start.elapsed().as_millis();
    log::info!(
        "Finished creating cubemap from equirectangular texture. Took {} ms",
        time
    );

    cubemap
}

#[derive(Copy, Clone)]
struct EnvPipelineParameters<'a> {
    viewport_info: &'a vk::PipelineViewportStateCreateInfo,
    rasterizer_info: &'a vk::PipelineRasterizationStateCreateInfo,
    dynamic_state_info: Option<&'a vk::PipelineDynamicStateCreateInfo>,
    layout: vk::PipelineLayout,
    format: vk::Format,
}

unsafe fn create_env_pipeline<V: Vertex>(
    device: &VkDevice,
    params: EnvPipelineParameters,
) -> vk::Pipeline {
    let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::_1)
        .min_sample_shading(1.0)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false);

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ZERO)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build()];

    create_pipeline::<V>(
        device,
        PipelineParameters {
            multisampling_info: &multisampling_info,
            viewport_info: params.viewport_info,
            rasterizer_info: params.rasterizer_info,
            dynamic_state_info: params.dynamic_state_info,
            depth_stencil_info: None,
            color_blend_attachments: &color_blend_attachments,
            layout: params.layout,
            render_pass: todo!(),
        },
    )
}

fn get_view_matrices() -> [Matrix4<f32>; 6] {
    [
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ),
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ),
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0),
        ),
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ),
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 1.0, 0.0),
        ),
        Matrix4::<f32>::look_at_rh(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 1.0, 0.0),
        ),
    ]
}
