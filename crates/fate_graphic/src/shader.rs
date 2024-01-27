use vulkanalia::vk::{PipelineShaderStageCreateInfoBuilder, ShaderModule};

use anyhow::Result;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Shader<'a> {
    vert_shader_module: ShaderModule,
    frag_shader_module: ShaderModule,
    vert_stage: PipelineShaderStageCreateInfoBuilder<'a>,
    frag_stage: PipelineShaderStageCreateInfoBuilder<'a>,
}

impl<'a> Shader<'a> {
    pub unsafe fn new(
        main_func_name: &'a [u8],
        device: &Device,
    ) -> Result<Self> {
        let vert = include_bytes!("../../../shaders/shader.vert.spv");
        let frag = include_bytes!("../../../shaders/shader.frag.spv");
        let vert_shader_module = create_shader_module(device, &vert[..])?;
        let frag_shader_module = create_shader_module(device, &frag[..])?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(main_func_name);
        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(main_func_name);

        Ok(Self {
            vert_shader_module,
            frag_shader_module,
            vert_stage,
            frag_stage,
        })
    }

    pub unsafe fn get_stages(
        &mut self,
    ) -> Result<(
        vk::PipelineShaderStageCreateInfoBuilder<'a>,
        vk::PipelineShaderStageCreateInfoBuilder<'a>,
    )> {
        Ok((self.vert_stage, self.frag_stage))
    }

    pub unsafe fn destory(&mut self, device: &Device) {
        device.destroy_shader_module(self.vert_shader_module, None);
        device.destroy_shader_module(self.frag_shader_module, None);
    }
}

unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode)?;

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}
