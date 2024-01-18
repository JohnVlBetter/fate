use anyhow::Result;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;

use crate::texture::create_image_view;
use crate::tools::QueueFamilyIndices;

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub unsafe fn get(instance: &Instance, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<Self> {
        Ok(Self {
            capabilities: instance.get_physical_device_surface_capabilities_khr(physical_device, surface)?,
            formats: instance.get_physical_device_surface_formats_khr(physical_device, surface)?,
            present_modes: instance.get_physical_device_surface_present_modes_khr(physical_device, surface)?,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Swapchain {
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub unsafe fn new(&mut self, width: u32, height: u32, instance: &Instance, 
        device: &Device, physical_device: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> Result<()> {
        // Image

        let indices = QueueFamilyIndices::get(instance, surface, physical_device)?;
        let support = SwapchainSupport::get(instance, physical_device, surface)?;

        let surface_format = Self::get_swapchain_surface_format(&support.formats);
        let present_mode = Self::get_swapchain_present_mode(&support.present_modes);
        let extent = Self::get_swapchain_extent(width, height, support.capabilities);

        self.swapchain_format = surface_format.format;
        self.swapchain_extent = extent;

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0 && image_count > support.capabilities.max_image_count {
            image_count = support.capabilities.max_image_count;
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if indices.graphics != indices.present {
            queue_family_indices.push(indices.graphics);
            queue_family_indices.push(indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        // Create

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        self.swapchain = device.create_swapchain_khr(&info, None)?;

        // Images

        self.swapchain_images = device.get_swapchain_images_khr(self.swapchain)?;

        Ok(())
    }

    pub unsafe fn create_swapchain_image_views(&mut self, device: &Device) -> Result<()> {
        self.swapchain_image_views = self
            .swapchain_images
            .iter()
            .map(|i| create_image_view(device, *i, self.swapchain_format, vk::ImageAspectFlags::COLOR, 1))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .cloned()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .unwrap_or_else(|| formats[0])
    }

    fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn get_swapchain_extent(width: u32, height: u32, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
            vk::Extent2D::builder()
                .width(clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                    width,
                ))
                .height(clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                    height,
                ))
                .build()
        }
    }
}