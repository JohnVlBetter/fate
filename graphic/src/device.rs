use std::collections::HashSet;

use anyhow::{anyhow, Result};
use log::*;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::CommandPool;
use vulkanalia::Version;

use crate::swapchain::SwapchainSupport;
use crate::tools::{QueueFamilyIndices, SuitabilityError};

/// Whether the validation layers should be enabled.
pub const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
/// The name of the validation layers.
pub const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

/// The required device extensions.
pub const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

/// The Vulkan SDK version that started requiring the portability subset extension for macOS.
pub const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

#[derive(Clone, Debug)]
pub struct VkDevice {
    // Physical Device / Logical Device
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub msaa_samples: vk::SampleCountFlags,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    // Command Pool
    pub command_pool: vk::CommandPool,
    // Command Buffers
    pub command_pools: Vec<vk::CommandPool>,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub secondary_command_buffers: Vec<Vec<vk::CommandBuffer>>,
}

impl VkDevice {
    pub unsafe fn new(entry: &Entry, instance: &Instance, surface: vk::SurfaceKHR) -> Self {
        let (physical_device, msaa_samples) = pick_physical_device(instance, surface).unwrap();
        let (device, graphics_queue, present_queue) =
            create_logical_device(entry, instance, surface, physical_device).unwrap();
        Self {
            device: device,
            physical_device: physical_device,
            msaa_samples: msaa_samples,
            graphics_queue: graphics_queue,
            present_queue: present_queue,
            command_pool: CommandPool::null(),
            command_pools: vec![],
            command_buffers: vec![],
            secondary_command_buffers: vec![],
        }
    }

    pub unsafe fn create_command_pools(
        &mut self,
        instance: &Instance,
        surface: vk::SurfaceKHR,
        num: usize,
    ) -> Result<()> {
        // Global

        self.command_pool =
            create_command_pool(instance, &self.device, surface, self.physical_device)?;

        // Per-framebuffer
        for _ in 0..num {
            let command_pool =
                create_command_pool(instance, &self.device, surface, self.physical_device)?;
            self.command_pools.push(command_pool);
        }

        Ok(())
    }

    pub unsafe fn destory(&mut self) -> (){
        self.command_pools.iter().for_each(|p| self.device.destroy_command_pool(*p, None));
        self.device.destroy_command_pool(self.command_pool, None);
        self.device.destroy_device(None);
    }

    pub unsafe fn destory_buffer(&mut self, buffer: vk::Buffer, buffer_memory: vk::DeviceMemory) ->() {
        self.device.free_memory(buffer_memory, None);
        self.device.destroy_buffer(buffer, None);
    }
}

unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(indices.graphics);

    Ok(device.create_command_pool(&info, None)?)
}

unsafe fn check_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, surface, physical_device)?;
    check_physical_device_extensions(instance, physical_device)?;

    let support = SwapchainSupport::get(instance, physical_device, surface)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy.")));
    }

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions."
        )))
    }
}

unsafe fn get_max_msaa_samples(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let properties = instance.get_physical_device_properties(physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}

unsafe fn pick_physical_device(
    instance: &Instance,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, vk::SampleCountFlags)> {
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, surface, physical_device) {
            warn!(
                "Skipping physical device (`{}`): {}",
                properties.device_name, error
            );
        } else {
            info!("Selected physical device (`{}`).", properties.device_name);
            let msaa_samples = get_max_msaa_samples(instance, physical_device);
            return Ok((physical_device, msaa_samples));
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}

unsafe fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Result<(Device, vk::Queue, vk::Queue)> {
    // Queue Create Infos
    let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    // Layers

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        vec![]
    };

    // Extensions

    let mut extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    // Required by Vulkan SDK on macOS since 1.3.216.
    if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
    }

    // Features

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);

    // Create

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let logic_device = instance.create_device(physical_device, &info, None)?;

    // Queues

    let graphics_queue = logic_device.get_device_queue(indices.graphics, 0);
    let present_queue = logic_device.get_device_queue(indices.present, 0);

    Ok((logic_device, graphics_queue, present_queue))
}
