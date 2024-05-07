use gltf::iter::Lights;
use gltf::khr_lights_punctual::{Kind, Light as GltfLight};
use gltf::Document;

#[derive(Copy, Clone, PartialEq, Debug)]
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
    pub fn new() -> Self {
        Light {
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: Some(1.0),
            light_type: LightType::DirectionalLight,
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

pub(crate) fn create_lights_from_gltf(document: &Document) -> Vec<Light> {
    document.lights().map_or(vec![], map_gltf_lights)
}
