/*use std::{ffi::CString, sync::Arc};
use vulkanalia::prelude::v1_0::*;

use crate::{device::VkDevice, shader::Shader, vertex::Vertex};

#[derive(Copy, Clone)]
pub struct PipelineParameters<'a> {
    pub vertex_shader_params: ShaderParameters<'a>,
    pub fragment_shader_params: ShaderParameters<'a>,
    pub multisampling_info: &'a vk::PipelineMultisampleStateCreateInfo,
    pub viewport_info: &'a vk::PipelineViewportStateCreateInfo,
    pub rasterizer_info: &'a vk::PipelineRasterizationStateCreateInfo,
    pub dynamic_state_info: Option<&'a vk::PipelineDynamicStateCreateInfo>,
    pub depth_stencil_info: Option<&'a vk::PipelineDepthStencilStateCreateInfo>,
    pub color_blend_attachments: &'a [vk::PipelineColorBlendAttachmentState],
    pub color_attachment_formats: &'a [vk::Format],
    pub depth_attachment_format: Option<vk::Format>,
    pub layout: vk::PipelineLayout,
    pub parent: Option<vk::Pipeline>,
    pub allow_derivatives: bool,
}

unsafe fn create_pipeline<V: Vertex>(device: &VkDevice, data: &mut AppData) -> Result<()> {
    // Shader
    let mut shader = Shader::new(b"main\0", &device.device)?;

    // Vertex Input State
    let binding_descriptions = V::binding_description();
    let attribute_descriptions = V::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    // Input Assembly State

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Viewport State

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain.swapchain_extent.width as f32)
        .height(data.swapchain.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(data.swapchain.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];
    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    // Rasterization State

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    // Multisample State

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(device.msaa_samples);

    // Depth Stencil State

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    // Color Blend State

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    // Push Constant Ranges

    let vert_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(64);

    let frag_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(64)
        .size(52);

    // Layout

    let set_layouts = &[data.descriptor_set_layout];
    let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges);

    data.pipeline_layout = device.device.create_pipeline_layout(&layout_info, None)?;

    // Create

    let (vert_stage, frag_stage) = shader.get_stages()?;
    let stages = &[vert_stage, frag_stage];
    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .multisample_state(&multisample_state)
        .depth_stencil_state(&depth_stencil_state)
        .color_blend_state(&color_blend_state)
        .layout(data.pipeline_layout)
        .render_pass(data.render_pass.render_pass)
        .subpass(0);

    data.pipeline = device
        .device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
        .0[0];

    // Cleanup

    shader.destory(&device.device);

    Ok(())
}

pub fn create_pipeline1<V: Vertex>(
    context: &Arc<Context>,
    params: PipelineParameters,
) -> vk::Pipeline {
    let entry_point_name = CString::new("main").unwrap();

    let (_vertex_shader_module, vertex_shader_state_info) = create_shader_stage_info(
        context,
        &entry_point_name,
        vk::ShaderStageFlags::VERTEX,
        params.vertex_shader_params,
    );

    let (_fragment_shader_module, fragment_shader_state_info) = create_shader_stage_info(
        context,
        &entry_point_name,
        vk::ShaderStageFlags::FRAGMENT,
        params.fragment_shader_params,
    );

    let shader_states_infos = [vertex_shader_state_info, fragment_shader_state_info];

    let bindings_descs = V::get_bindings_descriptions();
    let attributes_descs = V::get_attributes_descriptions();
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&bindings_descs)
        .vertex_attribute_descriptions(&attributes_descs);

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    let color_blending_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(params.color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let mut dynamic_rendering = vk::PipelineRenderingCreateInfo::builder()
        .color_attachment_formats(params.color_attachment_formats)
        .depth_attachment_format(params.depth_attachment_format.unwrap_or_default());

    let mut pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_states_infos)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_info)
        .viewport_state(params.viewport_info)
        .rasterization_state(params.rasterizer_info)
        .multisample_state(params.multisampling_info)
        .color_blend_state(&color_blending_info)
        .layout(params.layout)
        .push_next(&mut dynamic_rendering);

    if let Some(depth_stencil_info) = params.depth_stencil_info {
        pipeline_info = pipeline_info.depth_stencil_state(depth_stencil_info)
    }

    if let Some(dynamic_state_info) = params.dynamic_state_info {
        pipeline_info = pipeline_info.dynamic_state(dynamic_state_info);
    }

    if let Some(parent) = params.parent {
        pipeline_info = pipeline_info.base_pipeline_handle(parent);
    }

    if params.allow_derivatives {
        pipeline_info = pipeline_info.flags(vk::PipelineCreateFlags::ALLOW_DERIVATIVES);
    }

    let pipeline_info = pipeline_info.build();
    let pipeline_infos = [pipeline_info];

    unsafe {
        context
            .device()
            .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
            .expect("Failed to create graphics pipeline")[0]
    }
}

fn create_shader_stage_info(
    context: &Arc<Context>,
    entry_point_name: &CString,
    stage: vk::ShaderStageFlags,
    params: ShaderParameters,
) -> (ShaderModule, vk::PipelineShaderStageCreateInfo) {
    let extension = get_shader_file_extension(stage);
    let shader_path = format!("crates/viewer/shaders/{}.{}.spv", params.name, extension);
    let module = ShaderModule::new(Arc::clone(context), &shader_path);

    let mut stage_info = vk::PipelineShaderStageCreateInfo::builder()
        .stage(stage)
        .module(module.module())
        .name(entry_point_name);
    if let Some(specialization) = params.specialization {
        stage_info = stage_info.specialization_info(specialization);
    }
    let stage_info = stage_info.build();

    (module, stage_info)
}

fn get_shader_file_extension(stage: vk::ShaderStageFlags) -> &'static str {
    match stage {
        vk::ShaderStageFlags::VERTEX => "vert",
        vk::ShaderStageFlags::FRAGMENT => "frag",
        _ => panic!("Unsupported shader stage"),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ShaderParameters<'a> {
    name: &'a str,
    specialization: Option<&'a vk::SpecializationInfo>,
}

impl<'a> ShaderParameters<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            specialization: None,
        }
    }

    pub fn specialized(name: &'static str, specialization: &'a vk::SpecializationInfo) -> Self {
        Self {
            name,
            specialization: Some(specialization),
        }
    }
}
*/