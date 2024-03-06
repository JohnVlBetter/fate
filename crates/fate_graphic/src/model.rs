use crate::mesh::{create_meshes_from_gltf, Vec3};
use crate::texture::create_textures_from_gltf;
use crate::{mesh::Mesh, texture::Texture};
use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::device::VkDevice;
use crate::transform::Transform;

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub transform: Transform,
    pub textures: Vec<Texture>,
}

impl Model {
    pub unsafe fn new(path: &str, instance: &Instance, device: &VkDevice) -> Result<Self> {
        let (document, buffers, images) = gltf::import(path)?;

        let meshes = create_meshes_from_gltf(&document, &buffers, instance, device);

        let textures =
            create_textures_from_gltf(document.materials(), &images, instance, device);

        let transform = Transform::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        )?;

        Ok(Self {
            meshes,
            transform,
            textures,
        })
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        for texture in self.textures.iter_mut() {
            texture.destory(&device);
        }
    }
}
