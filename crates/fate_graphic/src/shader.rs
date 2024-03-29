use std::fs;

use vulkanalia::vk::{PipelineShaderStageCreateInfoBuilder, ShaderModule};

use anyhow::Result;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::prelude::v1_0::*;

pub struct Shader<'a> {
    device: &'a Device,
    pub vert_shader_module: ShaderModule,
    pub frag_shader_module: ShaderModule,
    pub vert_stage: PipelineShaderStageCreateInfoBuilder<'a>,
    pub frag_stage: PipelineShaderStageCreateInfoBuilder<'a>,
}

impl<'a> Shader<'a> {
    pub unsafe fn new(
        shader_name: String,
        main_func_name: &'a [u8],
        device: &'a Device,
    ) -> Result<Self> {
        let vert = fs::read(format!("shaders/{}.vert.spv", shader_name)).expect("读取失败！");
        let frag = fs::read(format!("shaders/{}.frag.spv", shader_name)).expect("读取失败！");
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
            device,
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

    pub unsafe fn destory(&mut self) {
        self.device
            .destroy_shader_module(self.vert_shader_module, None);
        self.device
            .destroy_shader_module(self.frag_shader_module, None);
    }
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        unsafe { self.destory() };
    }
}

unsafe fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode)?;

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.code_size())
        .code(bytecode.code());

    Ok(device.create_shader_module(&info, None)?)
}
