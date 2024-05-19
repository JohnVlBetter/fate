use crate::renderer::attachments::Attachments;
use crate::renderer::{fullscreen::*, RendererSettings};
use crate::FXAAMode;
use rendering::util::any_as_u8_slice;
use std::{mem::size_of, sync::Arc};
use vulkan::ash::{vk, Device};
use vulkan::{Context, Descriptors};

pub struct FXAAPass {
    context: Arc<Context>,
    descriptors: Descriptors,
    pipeline_layout: vk::PipelineLayout,
    quality_pipeline: vk::Pipeline,
    console_pipeline: vk::Pipeline,
    fxaa_mode: FXAAMode,
    absolute_luminance_threshold: f32,
    relative_luminance_threshold: f32,
    subpixel_blending: f32,
}

impl FXAAPass {
    pub fn create(
        context: Arc<Context>,
        output_format: vk::Format,
        attachments: &Attachments,
        settings: RendererSettings,
    ) -> Self {
        let descriptors = create_descriptors(&context, attachments);
        let pipeline_layout = create_pipeline_layout(context.device(), descriptors.layout());
        let quality_pipeline =
            create_pipeline(&context, output_format, pipeline_layout, FXAAMode::Quality);
        let console_pipeline =
            create_pipeline(&context, output_format, pipeline_layout, FXAAMode::Console);

        let fxaa_mode = settings.fxaa_mode;
        let absolute_luminance_threshold = settings.absolute_luminance_threshold;
        let relative_luminance_threshold = settings.relative_luminance_threshold;
        let subpixel_blending = settings.subpixel_blending;

        FXAAPass {
            context,
            descriptors,
            pipeline_layout,
            quality_pipeline,
            console_pipeline,
            fxaa_mode,
            absolute_luminance_threshold,
            relative_luminance_threshold,
            subpixel_blending,
        }
    }
}

impl FXAAPass {
    pub fn set_fxaa_mode(&mut self, fxaa_mode: FXAAMode) {
        self.fxaa_mode = fxaa_mode;
    }

    pub fn set_absolute_luminance_threshold(&mut self, absolute_luminance_threshold: f32) {
        self.absolute_luminance_threshold = absolute_luminance_threshold;
    }

    pub fn set_relative_luminance_threshold(&mut self, relative_luminance_threshold: f32) {
        self.relative_luminance_threshold = relative_luminance_threshold;
    }

    pub fn set_subpixel_blending(&mut self, subpixel_blending: f32) {
        self.subpixel_blending = subpixel_blending;
    }

    pub fn set_attachments(&mut self, attachments: &Attachments) {
        self.descriptors
            .sets()
            .iter()
            .for_each(|s| update_descriptor_set(&self.context, *s, attachments))
    }

    pub fn cmd_draw(&self, command_buffer: vk::CommandBuffer, quad_model: &QuadModel) {
        let device = self.context.device();
        let current_pipeline = match self.fxaa_mode {
            FXAAMode::Quality => self.quality_pipeline,
            FXAAMode::Console => self.console_pipeline,
        };

        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                current_pipeline,
            )
        };

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &[quad_model.vertices.buffer], &[0]);
            device.cmd_bind_index_buffer(
                command_buffer,
                quad_model.indices.buffer,
                0,
                vk::IndexType::UINT16,
            );
        }

        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                self.descriptors.sets(),
                &[],
            )
        };

        unsafe {
            let data = [
                self.absolute_luminance_threshold,
                self.relative_luminance_threshold,
                self.subpixel_blending,
            ];
            let data = any_as_u8_slice(&data);
            device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                data,
            );
        }

        unsafe { device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 1) };
    }
}

impl Drop for FXAAPass {
    fn drop(&mut self) {
        let device = self.context.device();
        unsafe {
            device.destroy_pipeline(self.quality_pipeline, None);
            device.destroy_pipeline(self.console_pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

fn create_descriptors(context: &Arc<Context>, attachments: &Attachments) -> Descriptors {
    let layout = create_descriptor_set_layout(context.device());
    let pool = create_descriptor_pool(context.device());
    let sets = create_descriptor_sets(context, pool, layout, attachments);
    Descriptors::new(Arc::clone(context), layout, pool, sets)
}

fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
    let bindings = [vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build()];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap()
    }
}
fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
    let pool_sizes = [vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
    }];

    let create_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(1)
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

    unsafe { device.create_descriptor_pool(&create_info, None).unwrap() }
}

fn create_descriptor_sets(
    context: &Arc<Context>,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    attachments: &Attachments,
) -> Vec<vk::DescriptorSet> {
    let layouts = [layout];
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    let sets = unsafe {
        context
            .device()
            .allocate_descriptor_sets(&allocate_info)
            .unwrap()
    };

    update_descriptor_set(context, sets[0], attachments);

    sets
}

fn update_descriptor_set(
    context: &Arc<Context>,
    set: vk::DescriptorSet,
    attachments: &Attachments,
) {
    let src_image_info = [vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(attachments.get_scene_resolved_color().view)
        .sampler(
            attachments
                .get_scene_resolved_color()
                .sampler
                .expect("FXAA Src Image没采样器"),
        )
        .build()];

    let descriptor_writes = [vk::WriteDescriptorSet::builder()
        .dst_set(set)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&src_image_info)
        .build()];

    unsafe {
        context
            .device()
            .update_descriptor_sets(&descriptor_writes, &[])
    }
}

fn create_pipeline_layout(
    device: &Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> vk::PipelineLayout {
    let layouts = [descriptor_set_layout];
    let push_constant_ranges = [vk::PushConstantRange {
        offset: 0,
        size: size_of::<f32>() as _,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
    }];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(&layouts)
        .push_constant_ranges(&push_constant_ranges);
    unsafe { device.create_pipeline_layout(&layout_info, None).unwrap() }
}

fn create_pipeline(
    context: &Arc<Context>,
    output_format: vk::Format,
    layout: vk::PipelineLayout,
    fxaa_mode: FXAAMode,
) -> vk::Pipeline {
    let (specialization_info, _map_entries, _data) =
        create_model_frag_shader_specialization(fxaa_mode);

    create_fullscreen_pipeline(
        context,
        output_format,
        layout,
        "fxaa",
        Some(&specialization_info),
    )
}
fn create_model_frag_shader_specialization(
    fxaa_mode: FXAAMode,
) -> (
    vk::SpecializationInfo,
    Vec<vk::SpecializationMapEntry>,
    Vec<u8>,
) {
    let map_entries = vec![vk::SpecializationMapEntry {
        constant_id: 0,
        offset: 0,
        size: size_of::<u32>(),
    }];

    let data = [fxaa_mode as u32];

    let data = Vec::from(unsafe { rendering::util::any_as_u8_slice(&data) });

    let specialization_info = vk::SpecializationInfo::builder()
        .map_entries(&map_entries)
        .data(&data)
        .build();

    (specialization_info, map_entries, data)
}
