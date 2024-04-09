use gltf::iter::Lights;
use gltf::khr_lights_punctual::{Kind, Light as GltfLight};
use gltf::Document;

#[derive(Copy, Clone, Debug)]
pub enum LightType {
    DirectionalLight,
    PointLight,
    SpotLight {
        inner_cone_angle: f32,
        outer_cone_angle: f32,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct Light {
    color: [f32; 3],
    intensity: f32,
    range: Option<f32>,
    light_type: LightType,
}

impl Light {
    pub unsafe fn new(
        color: [f32; 3],
        intensity: f32,
        range: Option<f32>,
        light_type: LightType,
    ) -> Self {
        Self {
            color,
            intensity,
            range,
            light_type,
        }
    }

    pub fn light_type(&self) -> LightType {
        self.light_type
    }

    pub fn color(&self) -> [f32; 3] {
        self.color
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    pub fn range(&self) -> Option<f32> {
        self.range
    }
}

fn map_gltf_lights(lights: Lights) -> Vec<Light> {
    lights
        .map(|light: GltfLight| -> Light {
            let light_type = match light.kind() {
                Kind::Directional => LightType::DirectionalLight,
                Kind::Point => LightType::PointLight,
                Kind::Spot {
                    inner_cone_angle,
                    outer_cone_angle,
                } => LightType::SpotLight {
                    inner_cone_angle,
                    outer_cone_angle,
                },
            };
            let color = light.color();
            let intensity = light.intensity();
            let range = light.range();

            Light {
                color,
                intensity,
                range,
                light_type,
            }
        })
        .collect()
}

pub fn create_lights_from_gltf(document: &Document) -> Vec<Light> {
    document.lights().map_or(vec![], map_gltf_lights)
}
