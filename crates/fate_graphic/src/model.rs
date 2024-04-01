use crate::aabb::Aabb;
use crate::mesh::create_meshes_from_gltf;
use crate::node::Nodes;
use crate::texture::create_textures_from_gltf;
use crate::{mesh::Mesh, texture::Texture};
use anyhow::Result;
use cgmath::Matrix4;
use vulkanalia::prelude::v1_0::*;

use crate::device::VkDevice;

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Texture>,
    nodes: Nodes,
    global_transform: Matrix4<f32>,
}

impl Model {
    pub unsafe fn new(path: &str, instance: &Instance, device: &VkDevice) -> Result<Self> {
        let (document, buffers, images) = gltf::import(path)?;

        let meshes = create_meshes_from_gltf(&document, &buffers, instance, device);

        let textures = create_textures_from_gltf(document.materials(), &images, instance, device);

        let scene = document
            .default_scene()
            .unwrap_or_else(|| document.scenes().next().unwrap());
        let mut nodes = Nodes::from_gltf_nodes(document.nodes(), &scene);
        let global_transform = {
            let aabb = compute_aabb(&nodes, &meshes);
            let transform = compute_unit_cube_at_origin_transform(aabb);
            nodes.transform(Some(transform));
            transform
        };

        Ok(Self {
            meshes,
            textures,
            nodes,
            global_transform,
        })
    }

    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }
    
    pub fn mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub unsafe fn destory(&mut self, device: &mut VkDevice) {
        for texture in self.textures.iter_mut() {
            texture.destory(&device);
        }
        for mesh in self.meshes.iter_mut() {
            for primitive in mesh.primitives.iter_mut() {
                primitive.destory(device);
            }
        }
    }
}

fn compute_aabb(nodes: &Nodes, meshes: &[Mesh]) -> Aabb<f32> {
    let aabbs = nodes
        .nodes()
        .iter()
        .filter(|n| n.mesh_index().is_some())
        .map(|n| {
            let mesh = &meshes[n.mesh_index().unwrap()];
            mesh.aabb() * n.transform()
        })
        .collect::<Vec<_>>();
    Aabb::union(&aabbs).unwrap()
}

fn compute_unit_cube_at_origin_transform(aabb: Aabb<f32>) -> Matrix4<f32> {
    let larger_side = aabb.larger_side_size();
    let scale_factor = (1.0_f32 / larger_side) * 10.0;

    let aabb = aabb * scale_factor;
    let center = aabb.center();

    let translation = Matrix4::from_translation(-center);
    let scale = Matrix4::from_scale(scale_factor);
    translation * scale
}
