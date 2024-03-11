use vulkanalia::prelude::v1_0::*;

use crate::texture::Texture;

pub struct Descriptors {
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
}

impl Descriptors {
    pub fn new(
        layout: vk::DescriptorSetLayout,
        pool: vk::DescriptorPool,
        sets: Vec<vk::DescriptorSet>,
    ) -> Self {
        Self { layout, pool, sets }
    }
}

impl Descriptors {
    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }

    pub fn pool(&self) -> vk::DescriptorPool {
        self.pool
    }

    pub fn sets(&self) -> &[vk::DescriptorSet] {
        &self.sets
    }

    pub fn set_sets(&mut self, sets: Vec<vk::DescriptorSet>) {
        self.sets = sets;
    }

    fn destory(&mut self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

pub unsafe fn create_descriptors(
    instance: &Instance,
    device: &Device,
    texture: &Texture,
) -> Descriptors {
    let bindings = [vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .build()];
    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
    let layout = device
        .create_descriptor_set_layout(&layout_info, None)
        .unwrap();

    let pool_sizes = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1);
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&[pool_sizes])
        .max_sets(1);
    let pool = device.create_descriptor_pool(&info, None).unwrap();

    let sets = {
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

        let cubemap_info = [vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(texture.view)
            .sampler(texture.sampler.unwrap())
            .build()];

        let descriptor_writes = [vk::WriteDescriptorSet::builder()
            .dst_set(sets[0])
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&cubemap_info)
            .build()];

        unsafe {
            context
                .device()
                .update_descriptor_sets(&descriptor_writes, &[])
        }

        sets
    };

    Descriptors::new(layout, pool, sets)
}
