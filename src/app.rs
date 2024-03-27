#![allow(
    dead_code,
    unused_variables,
    clippy::manual_slice_size_calculation,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use cgmath::{point3, vec3};
use fate_graphic::camera::Camera;
use fate_graphic::device::*;
use fate_graphic::frame_buffer::*;
use fate_graphic::light::Light;
use fate_graphic::material::PBRWorkflow;
use fate_graphic::material::MaterialUniform;
use fate_graphic::mesh;
use fate_graphic::mesh::Mat4;
use fate_graphic::mesh::ModelVertex;
use fate_graphic::mesh::Vec4;
use fate_graphic::model::*;
use fate_graphic::pipeline::create_pipeline;
use fate_graphic::pipeline::PipelineParameters;
use fate_graphic::render_pass::RenderPass;
use fate_graphic::swapchain::Swapchain;
use fate_graphic::texture::*;
use fate_graphic::uniform_buffer::UniformBuffer;
use fate_graphic::uniform_buffer::UniformBufferObject;
use std::collections::HashSet;
use std::ffi::CStr;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr::slice_from_raw_parts;
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
    pub camera: Camera,
    pub main_light: Light,
    pub model: Model,
}

pub fn simplify_path(path: String) -> String {
    let mut stack = Vec::new();
    path.split("/").for_each(|x| match x {
        "." | "" => (),
        ".." => {
            stack.pop();
        }
        _ => {
            stack.push(x);
        }
    });
    "/".to_string() + &stack.join("/")
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
        data.render_pass = RenderPass::new(&instance, &device, data.swapchain.swapchain_format)?;
        create_descriptor_set_layout(&device.device, &mut data)?;
        create_light_pass_pipeline(&device, &mut data)?;
        let num_images: usize = data.swapchain.swapchain_images.len();
        device.create_command_pools(&instance, data.surface, num_images)?;
        data.color_attachment = ColorAttachment::new(&instance, &device, &data.swapchain)?;
        data.depth_attachment = DepthAttachment::new(&instance, &device, &data.swapchain)?;
        create_framebuffers(&device.device, &mut data)?;
        let model = Model::new(
            "res/model/DamagedHelmet/glTF/DamagedHelmet.gltf",
            &instance,
            &device,
        )?;
        create_uniform_buffers(&instance, &device, &mut data)?;
        create_descriptor_pool(&device.device, &mut data)?;
        let dummy_texture = Texture::from_rgba(1, 1, &[std::u8::MAX; 4], true, &instance, &device).unwrap();
        create_descriptor_sets(&device.device, &mut data, &model, &dummy_texture)?;
        create_command_buffers(&mut device, &mut data)?;
        create_sync_objects(&device.device, &mut data)?;
        let camera = Camera::new(
            point3::<f32>(0.0, -5.0, 2.0),
            point3::<f32>(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            45.0,
            0.1,
            10.0,
        )?;
        let main_light = Light::new(
            mesh::Vec4::new(1.0, -1.0, -1.0, 1.0),
            mesh::Vec4::new(1.0, 1.0, 1.0, 1.0),
        )?;
        Ok(Self {
            entry,
            instance,
            data,
            device,
            frame: 0,
            resized: false,
            start: Instant::now(),
            models: 1,
            camera,
            main_light,
            model,
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

        //let model = self.model.transform.local_to_world_matrix();
        let mesh_nodes = self.model
                .nodes()
                .nodes()
                .iter()
                .filter(|n| n.mesh_index().is_some());
        let p_index = self.model.meshes[0].primitives()[0].index();
        let transforms = mesh_nodes.map(|n| n.transform()).collect::<Vec<_>>();
        let model = transforms[p_index];

        let model_bytes = &*slice_from_raw_parts(
            &model as *const Mat4 as *const u8,
            size_of::<Mat4>()
        );
        
        let mat_uniform = MaterialUniform::from(self.model.meshes[0].primitives()[0].material());
        let material_bytes = any_as_u8_slice(&mat_uniform);
        
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
        self.device.device.cmd_bind_vertex_buffers(command_buffer, 0, &[self.model.meshes[0].primitives()[0].vertex_buffer.buffer], &[0]);
        self.device.device.cmd_bind_index_buffer(command_buffer, self.model.meshes[0].primitives()[0].index_buffer.buffer, 0, vk::IndexType::UINT32);
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
            model_bytes.len() as u32,
            material_bytes,
        );
        self.device.device.cmd_draw_indexed(command_buffer, self.model.meshes[0].primitives()[0].indices.len() as u32, 1, 0, 0, 0);

        self.device.device.end_command_buffer(command_buffer)?;

        Ok(command_buffer)
    }

    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        // MVP

        let view = self.camera.get_view_mat();

        #[rustfmt::skip]
        let correction = Mat4::new(
            1.0,  0.0,       0.0, 0.0,
            0.0, -1.0,       0.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 0.0,
            0.0,  0.0, 1.0 / 2.0, 1.0,
        );

        let proj = correction
            * self.camera.get_proj_mat(
                self.data.swapchain.swapchain_extent.width as f32,
                self.data.swapchain.swapchain_extent.height as f32,
            );

        let color = Vec4::new(1.0, 0.0, 0.0, 1.0);

        let ubo = UniformBufferObject {
            view,
            proj,
            color,
            main_light_direction: self.main_light.direction,
            main_light_color: self.main_light.color,
            camera_pos: Vec4::new(self.camera.eye.x, self.camera.eye.y, self.camera.eye.z, 1.0),
        };

        self.data.uniform_buffers[image_index].update(&ubo, &self.device)?;

        Ok(())
    }

    #[rustfmt::skip]
    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device.device_wait_idle()?;
        self.destroy_swapchain();
        let size = window.inner_size();
        self.data.swapchain = Swapchain::new(size.width, size.height, &self.instance, &self.device.device, self.device.physical_device, self.data.surface)?;
        self.data.render_pass = RenderPass::new(&self.instance, &self.device, self.data.swapchain.swapchain_format)?;
        create_light_pass_pipeline(&self.device, &mut self.data)?;
        self.data.color_attachment = ColorAttachment::new(&self.instance, &self.device, &self.data.swapchain)?;
        self.data.depth_attachment = DepthAttachment::new(&self.instance, &self.device, &self.data.swapchain)?;
        create_framebuffers(&self.device.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device.device, &mut self.data)?;
        let dummy_texture = Texture::from_rgba(1, 1, &[std::u8::MAX; 4], true, &self.instance, &self.device).unwrap();
        create_descriptor_sets(&self.device.device, &mut self.data, &self.model, &dummy_texture)?;
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
        self.model.destory(&mut self.device);
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
        self.data.uniform_buffers.iter_mut().for_each(|b| b.destory(&self.device));
        self.data.depth_attachment.destory(&self.device);
        self.data.color_attachment.destory(&self.device);
        self.data.framebuffers.iter().for_each(|f| self.device.device.destroy_framebuffer(f.frame_buffer, None));
        self.device.device.destroy_pipeline(self.data.pipeline, None);
        self.device.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.data.render_pass.destory(&self.device);

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
    framebuffers: Vec<FrameBuffer>,
    // Color
    pub color_attachment: ColorAttachment,
    // Depth
    pub depth_attachment: DepthAttachment,
    //Buffer
    uniform_buffers: Vec<UniformBuffer>,
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

unsafe fn create_descriptor_set_layout(device: &Device, data: &mut AppData) -> Result<()> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(3)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(4)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(5)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
    ];

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

    data.descriptor_set_layout = device.create_descriptor_set_layout(&layout_info, None)?;

    Ok(())
}

unsafe fn create_light_pass_pipeline(device: &VkDevice, data: &mut AppData) -> Result<()> {
    // Multisample State
    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .rasterization_samples(device.msaa_samples);

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

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .build();
    let attachments = &[attachment];

    // Depth Stencil State
    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    // Push Constant Ranges
    let vert_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(64);

    let size = size_of::<MaterialUniform>() as u32;
    let frag_push_constant_range = vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .offset(64)
        .size(size);

    // Layout
    let set_layouts = &[data.descriptor_set_layout];
    let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(set_layouts)
        .push_constant_ranges(push_constant_ranges);

    data.pipeline_layout = device.device.create_pipeline_layout(&layout_info, None)?;

    data.pipeline = create_pipeline::<ModelVertex>(
        device,
        PipelineParameters {
            multisampling_info: &multisample_state,
            viewport_info: &viewport_state,
            rasterizer_info: &rasterization_state,
            dynamic_state_info: None,
            depth_stencil_info: Some(&depth_stencil_state),
            color_blend_attachments: attachments,
            layout: data.pipeline_layout,
            render_pass: data.render_pass.render_pass,
        },
    );
    
    Ok(())
}

unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> Result<()> {
    let count = data.swapchain.swapchain_image_views.len();
    data.framebuffers = vec![];
    let rc_color_attachment = Rc::new(data.color_attachment);
    let rc_depth_attachment = Rc::new(data.depth_attachment);
    for idx in 0..count {
        data.framebuffers.push(FrameBuffer::new(
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

unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &VkDevice,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_buffers.clear();

    for _ in 0..data.swapchain.swapchain_images.len() {
        data.uniform_buffers
            .push(UniformBuffer::new(instance, device)?);
    }

    Ok(())
}

unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain.swapchain_images.len() as u32 * 5);

    let pool_sizes = &[ubo_size, sampler_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain.swapchain_images.len() as u32);

    data.descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData, model: &Model,
    dummy_texture: &Texture) -> Result<()> {
    // Allocate

    let layouts = vec![data.descriptor_set_layout; data.swapchain.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    // Update

    for i in 0..data.swapchain.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i].buffer.buffer)
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let material = model.meshes[0].primitives()[0].material();

        let albedo_info = create_descriptor_image_info(
            &model.textures,
            material.albedo_texture_index(),
            dummy_texture
        );

        let normal_info = create_descriptor_image_info(
            &model.textures,
            material.normal_texture_index(),
            dummy_texture
        );

        let material_texture = match material.workflow() {
            PBRWorkflow::MetallicRoughness(workflow) => workflow.metallic_roughness_texture(),
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.specular_glossiness_texture(),
        };
        let material_info = create_descriptor_image_info(
            &model.textures,
            material_texture.map(|t| t.index()),
            dummy_texture
        );

        let ao_info = create_descriptor_image_info(
            &model.textures,
            material.ao_texture_index(),
            dummy_texture
        );

        let emissive_info = create_descriptor_image_info(
            &model.textures,
            material.emissive_texture_index(),
            dummy_texture
        );

        let albedo_sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&albedo_info);
        let normal_sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(2)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&normal_info);
        let material_sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(3)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&material_info);
        let ao_sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(4)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&ao_info);
        let emissive_sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(5)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&emissive_info);

        device.update_descriptor_sets(
            &[
                ubo_write,
                albedo_sampler_write,
                normal_sampler_write,
                material_sampler_write,
                ao_sampler_write,
                emissive_sampler_write,
            ],
            &[] as &[vk::CopyDescriptorSet],
        );
    }

    Ok(())
}

fn create_descriptor_image_info(
    textures: &[Texture],
    texture_idx: Option<usize>,
    dummy_texture: &Texture,
) -> [vk::DescriptorImageInfo; 1] {
    let (texture_image_view, texture_sampler) = texture_idx
        .map(|i| &textures[i])
        .map_or((dummy_texture.texture_image_view, dummy_texture.texture_sampler), |t| {
            (t.texture_image_view, t.texture_sampler)
        });
    [vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(texture_image_view)
        .sampler(texture_sampler)
        .build()]
}

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

pub unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}