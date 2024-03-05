use crate::mesh::{create_meshes_from_gltf, Vec3};
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
        
        let mut textures: Vec<Texture> = Vec::new();
        images.iter().enumerate().for_each(|(_index, image)| {
            let mut pixels = Vec::new();
            let size = image.width * image.height;
            for index in 0..size {
                let rgba = [
                    image.pixels[index as usize * 3],
                    image.pixels[index as usize * 3 + 1],
                    image.pixels[index as usize * 3 + 2],
                    255,
                ];
                pixels.extend_from_slice(&rgba);
            }
            let new_texture =
                Texture::new(pixels, image.width, image.height, instance, device).unwrap();
            textures.push(new_texture);
        });

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
