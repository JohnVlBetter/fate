use super::JointsBuffer;
use rendering::cgmath::{InnerSpace, Matrix4, SquareMatrix, Vector4};
use rendering::{
    light::{Light, LightType},
    material::{Material, PBRWorkflow},
    model::Model,
    skin::MAX_JOINTS_PER_MESH,
};
use std::{mem::size_of, sync::Arc};
use vulkan::{ash::vk, Buffer, Context};

const DEFAULT_LIGHT_DIRECTION: [f32; 4] = [0.0, 0.0, -1.0, 0.0];
const DIRECTIONAL_LIGHT_TYPE: u32 = 0;
const POINT_LIGHT_TYPE: u32 = 1;
const SPOT_LIGHT_TYPE: u32 = 2;
const NO_TEXTURE_ID: u32 = std::u8::MAX as u32;
const UNLIT_FLAG_LIT: u32 = 0;
const UNLIT_FLAG_UNLIT: u32 = 1;
const METALLIC_ROUGHNESS_WORKFLOW: u32 = 0;
const SPECULAR_GLOSSINESS_WORKFLOW: u32 = 1;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct LightUniform {
    light_space_matrix: Matrix4<f32>,
    position: [f32; 4],
    direction: [f32; 4],
    color: [f32; 4],
    intensity: f32,
    range: f32,
    angle_scale: f32,
    angle_offset: f32,
    light_type: u32,
    pad: [u32; 3],
}

impl From<(Matrix4<f32>, Light)> for LightUniform {
    fn from((transform, light): (Matrix4<f32>, Light)) -> Self {
        let position = [transform.w.x, transform.w.y, transform.w.z, 0.0];

        let direction = (transform * Vector4::from(DEFAULT_LIGHT_DIRECTION))
            .normalize()
            .into();

        let color = light.color();
        let color = [color[0], color[1], color[2], 0.0];

        let intensity = light.intensity();

        let range = light.range().unwrap_or(-1.0);

        let (angle_scale, angle_offset) = match light.light_type() {
            LightType::SpotLight {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                let outer_cos = outer_cone_angle.cos();
                let angle_scale =
                    1.0 / rendering::math::max(0.001, inner_cone_angle.cos() - outer_cos);
                let angle_offset = -outer_cos * angle_scale;
                (angle_scale, angle_offset)
            }
            _ => (-1.0, -1.0),
        };

        let light_type = match light.light_type() {
            LightType::DirectionalLight => DIRECTIONAL_LIGHT_TYPE,
            LightType::PointLight => POINT_LIGHT_TYPE,
            LightType::SpotLight { .. } => SPOT_LIGHT_TYPE,
        };

        let light_space_matrix = transform.invert().unwrap();

        Self {
            light_space_matrix,
            position,
            direction,
            color,
            intensity,
            range,
            angle_scale,
            angle_offset,
            light_type,
            pad: [0, 0, 0],
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct MaterialUniform {
    color: [f32; 4],
    // - emissive->[0,1,2] roughness/glossiness->[3]
    emissive_and_roughness_glossiness: [f32; 4],
    // - metallic->[0] (mr)
    // - specular->[0,1,2] (sg)
    // - occlusion->[3]
    metallic_specular_and_occlusion: [f32; 4],
    // [0-7] Color通道数
    // [8-15] metallic/roughness specular/glossiness通道数
    // [16-23] emissive通道数
    // [24-31] normals通道数
    color_material_emissive_normal_texture_channels: u32,
    // [0-7] Occlusion通道数
    // [8-15] Alpha mode
    // [16-23] Unlit flag
    // [24-31] Workflow
    occlusion_texture_channel_alpha_mode_unlit_flag_and_workflow: u32,
    alpha_cutoff: f32,
}

impl From<Material> for MaterialUniform {
    fn from(material: Material) -> MaterialUniform {
        let color = material.get_color();
        let emissive_factor = material.get_emissive();

        let workflow = material.get_workflow();

        let roughness_glossiness = match workflow {
            PBRWorkflow::MetallicRoughness(workflow) => workflow.get_roughness(),
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.get_glossiness(),
        };

        let emissive_and_roughness_glossiness = [
            emissive_factor[0],
            emissive_factor[1],
            emissive_factor[2],
            roughness_glossiness,
        ];

        let metallic_specular = match workflow {
            PBRWorkflow::MetallicRoughness(workflow) => [workflow.get_metallic(), 0.0, 0.0],
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.get_specular(),
        };

        let occlusion = material.get_occlusion();
        let metallic_specular_and_occlusion = [
            metallic_specular[0],
            metallic_specular[1],
            metallic_specular[2],
            occlusion,
        ];

        let color_texture_id = material
            .get_color_texture()
            .map_or(NO_TEXTURE_ID, |info| info.get_channel());

        let metallic_roughness_texture_id = match material.get_workflow() {
            PBRWorkflow::MetallicRoughness(workflow) => workflow.get_metallic_roughness_texture(),
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.get_specular_glossiness_texture(),
        }
        .map_or(NO_TEXTURE_ID, |t| t.get_channel());
        let emissive_texture_id = material
            .get_emissive_texture()
            .map_or(NO_TEXTURE_ID, |info| info.get_channel());
        let normal_texture_id = material
            .get_normals_texture()
            .map_or(NO_TEXTURE_ID, |info| info.get_channel());
        let color_material_emissive_normal_texture_channels = (color_texture_id << 24)
            | (metallic_roughness_texture_id << 16)
            | (emissive_texture_id << 8)
            | normal_texture_id;

        let occlusion_texture_id = material
            .get_occlusion_texture()
            .map_or(NO_TEXTURE_ID, |info| info.get_channel());
        let alpha_mode = material.get_alpha_mode();
        let unlit_flag = if material.is_unlit() {
            UNLIT_FLAG_UNLIT
        } else {
            UNLIT_FLAG_LIT
        };
        let workflow = if let PBRWorkflow::MetallicRoughness { .. } = workflow {
            METALLIC_ROUGHNESS_WORKFLOW
        } else {
            SPECULAR_GLOSSINESS_WORKFLOW
        };
        let occlusion_texture_channel_alpha_mode_unlit_flag_and_workflow =
            (occlusion_texture_id << 24) | (alpha_mode << 16) | (unlit_flag << 8) | workflow;

        let alpha_cutoff = material.get_alpha_cutoff();

        MaterialUniform {
            color,
            emissive_and_roughness_glossiness,
            metallic_specular_and_occlusion,
            color_material_emissive_normal_texture_channels,
            occlusion_texture_channel_alpha_mode_unlit_flag_and_workflow,
            alpha_cutoff,
        }
    }
}

pub fn create_transform_ubos(context: &Arc<Context>, model: &Model, count: u32) -> Vec<Buffer> {
    let mesh_node_count = model
        .nodes()
        .nodes()
        .iter()
        .filter(|n| n.mesh_index().is_some())
        .count() as u32;
    let elem_size = context.get_ubo_alignment::<Matrix4<f32>>();

    (0..count)
        .map(|_| {
            let mut buffer = Buffer::create(
                Arc::clone(context),
                u64::from(elem_size * mesh_node_count),
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            buffer.map_memory();
            buffer
        })
        .collect::<Vec<_>>()
}

pub fn create_skin_ubos(
    context: &Arc<Context>,
    model: &Model,
    count: u32,
) -> (Vec<Buffer>, Vec<Vec<JointsBuffer>>) {
    let skin_node_count = model.skins().len().max(1);
    let elem_size = context.get_ubo_alignment::<JointsBuffer>();

    let buffers = (0..count)
        .map(|_| {
            let mut buffer = Buffer::create(
                Arc::clone(context),
                u64::from(elem_size * skin_node_count as u32),
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            buffer.map_memory();
            buffer
        })
        .collect();

    let matrices = (0..count)
        .map(|_| {
            let mut matrices = Vec::with_capacity(skin_node_count);
            for _ in 0..skin_node_count {
                matrices.push([Matrix4::<f32>::identity(); MAX_JOINTS_PER_MESH]);
            }
            matrices
        })
        .collect();

    (buffers, matrices)
}

pub fn create_lights_ubos(context: &Arc<Context>, model: &Model, count: u32) -> Vec<Buffer> {
    let light_count = model
        .nodes()
        .nodes()
        .iter()
        .filter(|n| n.light_index().is_some())
        .count();

    //灯的数量不能为0
    let buffer_size = std::cmp::max(1, light_count) * size_of::<LightUniform>();

    (0..count)
        .map(|_| {
            Buffer::create(
                Arc::clone(context),
                buffer_size as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )
        })
        .collect::<Vec<_>>()
}
