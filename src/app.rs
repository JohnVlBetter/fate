#![allow(
    dead_code,
    unused_variables,
    clippy::manual_slice_size_calculation,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use cgmath::{point3, vec3, Deg};
use fate_graphic::buffer::*;
use fate_graphic::device::*;
use fate_graphic::framebuffer::*;
use fate_graphic::model::*;
use fate_graphic::render_pass::RenderPass;
use fate_graphic::shader::Shader;
use fate_graphic::swapchain::Swapchain;
use fate_graphic::texture::*;
use fate_graphic::tools::UniformBufferObject;
use std::collections::HashSet;
use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::{copy_nonoverlapping as memcpy, slice_from_raw_parts};
use std::rc::Rc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use log::*;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::window as vk_window;
use winit::window::Window;

use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[derive(Clone, Debug)]
pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: VkDevice,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
    pub models: usize,
}

impl App {
    pub unsafe fn new(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        let mut device = VkDevice::new(&entry, &instance, data.surface)?;
        let size = window.inner_size();
        data.swapchain = Swapchain::new(
            size.width,
            size.height,
            &instance,
            &device.device,
            device.physical_device,
            data.surface,
        )?;
        data.render_pass = RenderPass::new(&instance, &device, &data.swapchain)?;
        create_descriptor_set_layout(&device.device, &mut data)?;
        create_pipeline(&device, &mut data)?;
        let num_images: usize = data.swapchain.swapchain_images.len();
        device.create_command_pools(&instance, data.surface, num_images)?;
        data.color_attachment = ColorAttachment::new(&instance, &device, &data.swapchain)?;
        data.depth_attachment = DepthAttachment::new(&instance, &device, &data.swapchain)?;
        create_framebuffers(&device.device, &mut data)?;
        data.texture = Texture::new("res/model/viking_room/viking_room.png", &instance, &device)?;
        data.model = Model::new("res/model/viking_room/viking_room.obj", &instance, &device)?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device.device, &mut data)?;
        create_descriptor_sets(&device.device, &mut data)?;
        create_command_buffers(&mut device, &mut data)?;
        create_sync_objects(&device.device, &mut data)?;
        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
            models: 1,
        })
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];

        self.device
            .device
            .wait_for_fences(&[in_flight_fence], true, u64::max_value())?;

        let result = self.device.device.acquire_next_image_khr(
            self.data.swapchain.swapchain,
            u64::max_value(),
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        let image_in_flight = self.data.images_in_flight[image_index];
        if !image_in_flight.is_null() {
            self.device
                .device
                .wait_for_fences(&[image_in_flight], true, u64::max_value())?;
        }

        self.data.images_in_flight[image_index] = in_flight_fence;

        self.update_command_buffer(image_index)?;
        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.device.command_buffers[image_index]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        self.device.device.reset_fences(&[in_flight_fence])?;

        self.device.device.queue_submit(
            self.device.graphics_queue,
            &[submit_info],
            in_flight_fence,
        )?;

        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self
            .device
            .device
            .queue_present_khr(self.device.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    #[rustfmt::skip]
    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        // Reset

        let command_pool = self.device.command_pools[image_index];
        self.device.device.reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.device.command_buffers[image_index];

        // Commands

        let info = vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.device.device.begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass.render_pass)
            .framebuffer(self.data.framebuffers[image_index].frame_buffer)
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.device.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS);

        let secondary_command_buffers = (0..self.models)
            .map(|i| self.update_secondary_command_buffer(image_index, i))
            .collect::<Result<Vec<_>, _>>()?;
        self.device.device.cmd_execute_commands(command_buffer, &secondary_command_buffers[..]);

        self.device.device.cmd_end_render_pass(command_buffer);

        self.device.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    #[rustfmt::skip]
    unsafe fn update_secondary_command_buffer(
        &mut self,
        image_index: usize,
        model_index: usize,
    ) -> Result<vk::CommandBuffer> {
        // Allocate

        let command_buffers = &mut self.device.secondary_command_buffers[image_index];
        while model_index >= command_buffers.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.device.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = self.device.device.allocate_command_buffers(&allocate_info)?[0];
            command_buffers.push(command_buffer);
        }

        let command_buffer = command_buffers[model_index];

        // Model

        let y = (((model_index % 3) as f32) * 1.5) - 1.25;
        let z = (((model_index / 3) as f32) * -1.0) + 1.0;

        let time = self.start.elapsed().as_secs_f32();

        let model = Mat4::from_translation(vec3(0.0, y, z)) * Mat4::from_axis_angle(
            vec3(0.0, 0.0, 1.0),
            Deg(90.0) * time
        );

        let model_bytes = &*slice_from_raw_parts(
            &model as *const Mat4 as *const u8,
            size_of::<Mat4>()
        );

        let opacity = (model_index + 1) as f32 * 0.2;
        let opacity_bytes = &opacity.to_ne_bytes()[..];

        // Commands

        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.render_pass.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index].frame_buffer);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        self.device.device.begin_command_buffer(command_buffer, &info)?;

        self.device.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline);
        self.device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.data.model.vertex_buffer.buffer], &[0]);
        self.device.device.cmd_bind_index_buffer(command_buffer, self.data.model.index_buffer.buffer, 0, vk::IndexType::UINT32);
        self.device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline_layout,
            0,
            &[self.data.descriptor_sets[image_index]],
            &[],
        );
        self.device.device.cmd_push_constants(
            command_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );
        self.device.device.cmd_push_constants(
            command_buffer,
            self.data.pipeline_layout,
            vk::ShaderStageFlags::FRAGMENT,
            64,
            opacity_bytes,
        );
        self.device.device.cmd_draw_indexed(command_buffer, self.data.model.indices.len() as u32, 1, 0, 0, 0);

        self.device.device.end_command_buffer(command_buffer)?;

        Ok(command_buffer)
    }

    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        // MVP

        let view = Mat4::look_at_rh(
            point3::<f32>(6.0, 0.0, 2.0),
            point3::<f32>(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
        );

        #[rustfmt::skip]
        let correction = Mat4::new(
            1.0,  0.0,       0.0, 0.0,
            0.0, -1.0,       0.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 1.0,
        );

        let proj = correction
            * cgmath::perspective(
                Deg(45.0),
                self.data.swapchain.swapchain_extent.width as f32
                    / self.data.swapchain.swapchain_extent.height as f32,
                0.1,
                10.0,
            );

        let ubo = UniformBufferObject { view, proj };

        // Copy

        let memory = self.device.device.map_memory(
            self.data.uniform_buffers[image_index].buffer_memory,
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(&ubo, memory.cast(), 1);

        self.device
            .device
            .unmap_memory(self.data.uniform_buffers[image_index].buffer_memory);

        Ok(())
    }

    #[rustfmt::skip]
    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device.device_wait_idle()?;
        self.destroy_swapchain();
        let size = window.inner_size();
        self.data.swapchain = Swapchain::new(size.width, size.height, &self.instance, &self.device.device, self.device.physical_device, self.data.surface)?;
        self.data.render_pass = RenderPass::new(&self.instance, &self.device, &self.data.swapchain)?;
        create_pipeline(&self.device, &mut self.data)?;
        self.data.color_attachment = ColorAttachment::new(&self.instance, &self.device, &self.data.swapchain)?;
        self.data.depth_attachment = DepthAttachment::new(&self.instance, &self.device, &self.data.swapchain)?;
        create_framebuffers(&self.device.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device.device, &mut self.data)?;
        create_descriptor_sets(&self.device.device, &mut self.data)?;
        create_command_buffers(&mut self.device, &mut self.data)?;
        self.data.images_in_flight.resize(self.data.swapchain.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    #[rustfmt::skip]
    pub unsafe fn destroy(&mut self) {
        self.device.device.device_wait_idle().unwrap();

        self.destroy_swapchain();

        self.data.in_flight_fences.iter().for_each(|f| self.device.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores.iter().for_each(|s| self.device.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores.iter().for_each(|s| self.device.device.destroy_semaphore(*s, None));
        self.device.destory_buffer(&self.data.model.index_buffer);
        self.device.destory_buffer(&self.data.model.vertex_buffer);
        self.data.texture.destory(&self.device);
        self.device.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);
        self.device.destory();
        self.instance.destroy_surface_khr(self.data.surface, None);

        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }

    #[rustfmt::skip]
    unsafe fn destroy_swapchain(&mut self) {
        self.device.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data.uniform_buffers.iter().for_each(|m| self.device.device.free_memory(m.buffer_memory, None));
        self.data.uniform_buffers.iter().for_each(|b| self.device.device.destroy_buffer(b.buffer, None));
        self.data.depth_attachment.destory(&self.device);
        self.data.color_attachment.destory(&self.device);
        self.data.framebuffers.iter().for_each(|f| self.device.device.destroy_framebuffer(f.frame_buffer, None));
        self.device.device.destroy_pipeline(self.data.pipeline, None);
        self.device.device.destroy_pipeline_layout(self.data.pipeline_layout, None);

        self.data.swapchain.destroy(&self.device);
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppData {
    // Debug
    messenger: vk::DebugUtilsMessengerEXT,
    // Surface
    surface: vk::SurfaceKHR,
    // Pipeline
    pub render_pass: RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    // Framebuffers
    framebuffers: Vec<Framebuffer>,
    // Color
    pub color_attachment: ColorAttachment,
    // Depth
    pub depth_attachment: DepthAttachment,
    //Texture
    pub texture: Texture,
    // Model
    model: Model,
    //Buffer
    uniform_buffers: Vec<Buffer>,
    //Swapchain
    pub swapchain: Swapchain,
    // Descriptors
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    // Sync Objects
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
}

unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {
    // Application Info

    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Fate Launcher\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"Fate Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    // Layers

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    // Extensions

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    // Required by Vulkan SDK on macOS since 1.3.216.
    let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");
        extensions.push(
            vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                .name
                .as_ptr(),
        );
        extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::empty()
    };

    if VALIDATION_ENABLED {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Create

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
        .user_callback(Some(debug_callback));

    if VALIDATION_ENABLED {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    // Messenger

    if VALIDATION_ENABLED {
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}

//================================================
// Pipeline
//================================================

unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    data.descriptor_set_layout = device.create_descriptor_set_layout(&info, None)?;

    Ok(())
}

unsafe fn create_pipeline(device: &VkDevice, data: &mut AppData) -> Result<()> {
    // Shader
    let mut shader = Shader::new(b"main\0", &device.device)?;

    // Vertex Input State
    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_descriptions();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
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
        .cull_mode(vk::CullModeFlags::BACK)
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
        .size(4);

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

//================================================
// Framebuffers
//================================================

unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> Result<()> {
    let count = data.swapchain.swapchain_image_views.len();
    data.framebuffers = vec![];
    let rc_color_attachment = Rc::new(data.color_attachment);
    let rc_depth_attachment = Rc::new(data.depth_attachment);
    for idx in 0..count {
        data.framebuffers.push(Framebuffer::new(
            device,
            &data.swapchain,
            &rc_color_attachment,
            &rc_depth_attachment,
            data.render_pass.render_pass,
            idx,
        )?);
    }
    drop(rc_depth_attachment);
    drop(rc_color_attachment);
    Ok(())
}

//================================================
// Buffers
//================================================

unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &VkDevice,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_buffers.clear();

    for _ in 0..data.swapchain.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            &device.device,
            device.physical_device,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(Buffer {
            buffer: uniform_buffer,
            buffer_memory: uniform_buffer_memory,
        });
    }

    Ok(())
}

//================================================
// Descriptors
//================================================

unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain.swapchain_images.len() as u32);

    data.descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData) -> Result<()> {
    // Allocate

    let layouts = vec![data.descriptor_set_layout; data.swapchain.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..data.swapchain.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i].buffer)
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture.texture_image_view)
            .sampler(data.texture.texture_sampler);

        let image_info = &[info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_info);

        device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(())
}

//================================================
// Command Buffers
//================================================

unsafe fn create_command_buffers(device: &mut VkDevice, data: &mut AppData) -> Result<()> {
    let num_images = data.swapchain.swapchain_images.len();
    for image_index in 0..num_images {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(device.command_pools[image_index])
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.device.allocate_command_buffers(&allocate_info)?[0];
        device.command_buffers.push(command_buffer);
    }

    device.secondary_command_buffers = vec![vec![]; data.swapchain.swapchain_images.len()];

    Ok(())
}

//================================================
// Sync Objects
//================================================

unsafe fn create_sync_objects(device: &Device, data: &mut AppData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        data.image_available_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);
        data.render_finished_semaphores
            .push(device.create_semaphore(&semaphore_info, None)?);

        data.in_flight_fences
            .push(device.create_fence(&fence_info, None)?);
    }

    data.images_in_flight = data
        .swapchain
        .swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}
