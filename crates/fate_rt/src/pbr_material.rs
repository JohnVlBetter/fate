use cgmath::{InnerSpace, Vector3};
use gltf::{material::Material as GltfMaterial, texture::Info};
use std::{f64::consts::PI, sync::Arc};

use crate::{
    hit::HitRecord,
    material::{Scatter, ScatterRecord},
    mesh::{Vec3, Vec4},
    pdf::CosinePdf,
    ray::Ray,
    texture::Texture,
};

#[derive(Clone)]
pub enum PBRWorkflow {
    MetallicRoughness(MetallicRoughness),
    SpecularGlossiness(SpecularGlossiness),
}

#[derive(Clone)]
pub struct MetallicRoughness {
    metallic_roughness_texture: Option<Arc<dyn Texture>>,
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

    pub fn metallic_roughness_texture(&self) -> Option<Arc<dyn Texture>> {
        self.metallic_roughness_texture
    }
}

#[derive(Clone)]
pub struct SpecularGlossiness {
    specular_glossiness_texture: Option<Arc<dyn Texture>>,
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

    pub fn specular_glossiness_texture(&self) -> Option<Arc<dyn Texture>> {
        self.specular_glossiness_texture
    }
}

pub struct PBRMaterial {
    albedo_texture: Option<Arc<dyn Texture>>,
    emissive_texture: Option<Arc<dyn Texture>>,
    //normal_texture: Option<Arc<dyn Texture>>,
    //ao_texture: Option<Arc<dyn Texture>>,
    //workflow: PBRWorkflow,
    base_color: Vec4,
    emissive: Vec3,
    //occlusion: f32,
    //double_sided: bool,
    is_unlit: bool,
}

impl PBRMaterial {
    fn from(material: GltfMaterial) -> PBRMaterial {
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

        let is_unlit = material.unlit();

        PBRMaterial {
            base_color,
            emissive,
            albedo_texture,
            emissive_texture,
            is_unlit,
        }
    }
}

impl Scatter for PBRMaterial {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self
            .albedo_texture
            .map(|tex| tex.value(rec.u, rec.v, rec.p))
            .unwrap();
        srec.pdf = Box::new(CosinePdf::new(rec.normal));
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = Vector3::dot(rec.normal, scattered.direction().normalize());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }
}
