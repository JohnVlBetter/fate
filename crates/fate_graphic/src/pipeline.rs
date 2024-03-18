use vulkanalia::prelude::v1_0::*;

use crate::{device::VkDevice, shader::Shader, vertex::Vertex};

#[derive(Copy, Clone)]
pub struct PipelineParameters<'a> {
    pub multisampling_info: &'a vk::PipelineMultisampleStateCreateInfo,
    pub viewport_info: &'a vk::PipelineViewportStateCreateInfo,
    pub rasterizer_info: &'a vk::PipelineRasterizationStateCreateInfo,
    pub dynamic_state_info: Option<&'a vk::PipelineDynamicStateCreateInfo>,
    pub depth_stencil_info: Option<&'a vk::PipelineDepthStencilStateCreateInfo>,
    pub color_blend_attachments: &'a [vk::PipelineColorBlendAttachmentState],
    pub render_pass: vk::RenderPass,
    pub layout: vk::PipelineLayout,
}

pub unsafe fn create_pipeline<V: Vertex>(
    device: &VkDevice,
    params: PipelineParameters,
) -> vk::Pipeline {
    // Shader
    let mut shader = Shader::new(String::from("shader"), b"main\0", &device.device).unwrap();
    let (vert_stage, frag_stage) = shader.get_stages().unwrap();
    let stages = &[vert_stage, frag_stage];

    // Vertex Input State
    let binding_descriptions = V::binding_description();
    let attribute_descriptions = V::attribute_descriptions();
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(&binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    // Input Assembly State
    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Color Blend State
    let color_blending_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(params.color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let mut pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_state)
        .viewport_state(params.viewport_info)
        .rasterization_state(params.rasterizer_info)
        .multisample_state(params.multisampling_info)
        .color_blend_state(&color_blending_info)
        .layout(params.layout)
        .render_pass(params.render_pass)
        .subpass(0);

    if let Some(depth_stencil_info) = params.depth_stencil_info {
        pipeline_info = pipeline_info.depth_stencil_state(depth_stencil_info)
    }

    if let Some(dynamic_state_info) = params.dynamic_state_info {
        pipeline_info = pipeline_info.dynamic_state(dynamic_state_info);
    }

    let pipeline_info = pipeline_info.build();

    let pipeline = device
        .device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .unwrap()
        .0[0];

    shader.destory(&device.device);

    pipeline
}
