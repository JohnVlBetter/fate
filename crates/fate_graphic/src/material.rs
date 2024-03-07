use crate::mesh::{Vec3, Vec4};
use gltf::{
    material::{AlphaMode, Material as GltfMaterial, NormalTexture, OcclusionTexture},
    texture::Info,
};

const ALPHA_MODE_OPAQUE: u32 = 0;
const ALPHA_MODE_MASK: u32 = 1;
const ALPHA_MODE_BLEND: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub struct TextureInfo {
    index: usize,
    channel: u32,
}

impl TextureInfo {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn channel(&self) -> u32 {
        self.channel
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PBRWorkflow {
    MetallicRoughness(MetallicRoughness),
    SpecularGlossiness(SpecularGlossiness),
}

#[derive(Clone, Copy, Debug)]
pub struct MetallicRoughness {
    metallic_roughness_texture: Option<TextureInfo>,
    metallic: f32,
    roughness: f32,
}

impl MetallicRoughness {
    pub fn metallic(&self) -> f32 {
        self.metallic
    }

    pub fn roughness(&self) -> f32 {
        self.roughness
    }

    pub fn metallic_roughness_texture(&self) -> Option<TextureInfo> {
        self.metallic_roughness_texture
    }

    pub fn texture_index(&self) -> Option<usize> {
        self.metallic_roughness_texture.map(|info| info.index)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpecularGlossiness {
    specular_glossiness_texture: Option<TextureInfo>,
    specular: Vec3,
    glossiness: f32,
}

impl SpecularGlossiness {
    pub fn specular(&self) -> Vec3 {
        self.specular
    }

    pub fn glossiness(&self) -> f32 {
        self.glossiness
    }

    pub fn specular_glossiness_texture(&self) -> Option<TextureInfo> {
        self.specular_glossiness_texture
    }

    pub fn texture_index(&self) -> Option<usize> {
        self.specular_glossiness_texture.map(|info| info.index)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    albedo_texture: Option<TextureInfo>,
    emissive_texture: Option<TextureInfo>,
    normal_texture: Option<TextureInfo>,
    ao_texture: Option<TextureInfo>,
    workflow: PBRWorkflow,
    base_color: Vec4,
    emissive: Vec3,
    occlusion: f32,
    alpha_mode: u32,
    alpha_cutoff: f32,
    double_sided: bool,
    is_unlit: bool,
}

pub struct MaterialUniform {
    pub base_color: Vec4,
    pub emissive_and_roughness_glossiness: Vec4,
    pub metallic_specular_and_occlusion: Vec4,
    pub workflow: u32,
}

impl From<Material> for MaterialUniform {
    fn from(material: Material) -> MaterialUniform {
        let base_color = material.base_color();
        let emissive_factor = material.emissive();

        let workflow = material.workflow();

        let roughness_glossiness = match workflow {
            PBRWorkflow::MetallicRoughness(workflow) => workflow.roughness(),
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.glossiness(),
        };

        let emissive_and_roughness_glossiness = Vec4::new(
            emissive_factor[0],
            emissive_factor[1],
            emissive_factor[2],
            roughness_glossiness,
        );

        let metallic_specular = match workflow {
            PBRWorkflow::MetallicRoughness(workflow) => Vec3::new(workflow.metallic(), 0.0, 0.0),
            PBRWorkflow::SpecularGlossiness(workflow) => workflow.specular(),
        };

        let occlusion = material.occlusion();
        let metallic_specular_and_occlusion = Vec4::new(
            metallic_specular[0],
            metallic_specular[1],
            metallic_specular[2],
            occlusion,
        );

        const METALLIC_ROUGHNESS_WORKFLOW: u32 = 0;
        const SPECULAR_GLOSSINESS_WORKFLOW: u32 = 1;
        let workflow = if let PBRWorkflow::MetallicRoughness { .. } = workflow {
            METALLIC_ROUGHNESS_WORKFLOW
        } else {
            SPECULAR_GLOSSINESS_WORKFLOW
        };

        MaterialUniform {
            base_color,
            emissive_and_roughness_glossiness,
            metallic_specular_and_occlusion,
            workflow,
        }
    }
}

impl Material {
    pub fn base_color(&self) -> Vec4 {
        self.base_color
    }

    pub fn emissive(&self) -> Vec3 {
        self.emissive
    }

    pub fn occlusion(&self) -> f32 {
        self.occlusion
    }

    pub fn alpha_mode(&self) -> u32 {
        self.alpha_mode
    }

    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }

    pub fn is_double_sided(&self) -> bool {
        self.double_sided
    }

    pub fn albedo_texture(&self) -> Option<TextureInfo> {
        self.albedo_texture
    }

    pub fn emissive_texture(&self) -> Option<TextureInfo> {
        self.emissive_texture
    }

    pub fn normal_texture(&self) -> Option<TextureInfo> {
        self.normal_texture
    }

    pub fn ao_texture(&self) -> Option<TextureInfo> {
        self.ao_texture
    }

    pub fn is_transparent(&self) -> bool {
        self.alpha_mode == ALPHA_MODE_BLEND
    }

    pub fn albedo_texture_index(&self) -> Option<usize> {
        self.albedo_texture.map(|info| info.index)
    }

    pub fn emissive_texture_index(&self) -> Option<usize> {
        self.emissive_texture.map(|info| info.index)
    }

    pub fn normal_texture_index(&self) -> Option<usize> {
        self.normal_texture.map(|info| info.index)
    }

    pub fn ao_texture_index(&self) -> Option<usize> {
        self.ao_texture.map(|info| info.index)
    }

    pub fn is_unlit(&self) -> bool {
        self.is_unlit
    }

    pub fn workflow(&self) -> PBRWorkflow {
        self.workflow
    }
}

impl<'a> From<GltfMaterial<'a>> for Material {
    fn from(material: GltfMaterial) -> Material {
        let base_color = match material.pbr_specular_glossiness() {
            Some(pbr) => pbr.diffuse_factor(),
            _ => material.pbr_metallic_roughness().base_color_factor(),
        };
        let base_color = Vec4::new(base_color[0], base_color[1], base_color[2], base_color[3]);

        let emissive_strength = material.emissive_strength().unwrap_or(1.0);
        let emissive = material.emissive_factor();
        let emissive = Vec3::new(
            emissive[0] * emissive_strength,
            emissive[1] * emissive_strength,
            emissive[2] * emissive_strength,
        );

        let albedo_texture = match material.pbr_specular_glossiness() {
            Some(pbr) => pbr.diffuse_texture(),
            _ => material.pbr_metallic_roughness().base_color_texture(),
        };
        let albedo_texture = get_texture(albedo_texture);
        let emissive_texture = get_texture(material.emissive_texture());
        let normal_texture = normal_texture(material.normal_texture());
        let (occlusion, ao_texture) = get_occlusion(material.occlusion_texture());

        let workflow = match material.pbr_specular_glossiness() {
            Some(pbr) => PBRWorkflow::SpecularGlossiness(SpecularGlossiness {
                specular: Vec3::new(
                    pbr.specular_factor()[0],
                    pbr.specular_factor()[1],
                    pbr.specular_factor()[2],
                ),
                glossiness: pbr.glossiness_factor(),
                specular_glossiness_texture: get_texture(pbr.specular_glossiness_texture()),
            }),
            _ => {
                let pbr = material.pbr_metallic_roughness();
                PBRWorkflow::MetallicRoughness(MetallicRoughness {
                    metallic: pbr.metallic_factor(),
                    roughness: pbr.roughness_factor(),
                    metallic_roughness_texture: get_texture(pbr.metallic_roughness_texture()),
                })
            }
        };

        let alpha_mode = alpha_mode_index(material.alpha_mode());
        let alpha_cutoff = material.alpha_cutoff().unwrap_or(0.5);

        let double_sided = material.double_sided();

        let is_unlit = material.unlit();

        Material {
            base_color,
            emissive,
            occlusion,
            albedo_texture,
            emissive_texture,
            normal_texture,
            ao_texture,
            workflow,
            alpha_mode,
            alpha_cutoff,
            double_sided,
            is_unlit,
        }
    }
}

fn get_texture(texture_info: Option<Info>) -> Option<TextureInfo> {
    texture_info.map(|tex_info| TextureInfo {
        index: tex_info.texture().index(),
        channel: tex_info.tex_coord(),
    })
}

fn normal_texture(texture_info: Option<NormalTexture>) -> Option<TextureInfo> {
    texture_info.map(|tex_info| TextureInfo {
        index: tex_info.texture().index(),
        channel: tex_info.tex_coord(),
    })
}

fn get_occlusion(texture_info: Option<OcclusionTexture>) -> (f32, Option<TextureInfo>) {
    let strength = texture_info
        .as_ref()
        .map_or(0.0, |tex_info| tex_info.strength());

    let texture = texture_info.map(|tex_info| TextureInfo {
        index: tex_info.texture().index(),
        channel: tex_info.tex_coord(),
    });

    (strength, texture)
}

fn alpha_mode_index(alpha_mode: AlphaMode) -> u32 {
    match alpha_mode {
        AlphaMode::Opaque => ALPHA_MODE_OPAQUE,
        AlphaMode::Mask => ALPHA_MODE_MASK,
        AlphaMode::Blend => ALPHA_MODE_BLEND,
    }
}
