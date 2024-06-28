use crate::mesh::{create_meshes_from_gltf, Mesh, Meshes};
use cgmath::{Vector3, Zero};
use gltf::image::Source;
use gltf::{iter::Nodes as GltfNodes, Scene};
use rendering::{
    animation::{load_animations, Animations, PlaybackMode, PlaybackState},
    error::ModelLoadingError,
    light::{create_lights_from_gltf, Light},
    metadata::Metadata,
    skin::{create_skins_from_gltf, Skin},
    texture::{self, Texture, Textures},
    Aabb,
};
use scene::scene_tree::Node;
use scene::transform::Transform;
use std::{error::Error, path::Path, rc::Rc, result::Result, sync::Arc};
use vulkan::{ash::vk, Buffer, Context, PreLoadedResource};

pub struct ModelStagingResources {
    _staged_vertices: Buffer,
    _staged_indices: Option<Buffer>,
    _staged_textures: Vec<Buffer>,
}

pub struct Model {
    metadata: Metadata,
    meshes: Vec<Mesh>,
    node: Rc<Node>,
    animations: Option<Animations>,
    skins: Vec<Skin>,
    textures: Textures,
    lights: Vec<Light>,
    transform: Transform,
}

impl Model {
    pub fn create_from_file<P: AsRef<Path>>(
        context: Arc<Context>,
        command_buffer: vk::CommandBuffer,
        path: P,
    ) -> Result<PreLoadedResource<Model, ModelStagingResources>, Box<dyn Error>> {
        let (document, buffers, images) = gltf::import(&path)?;

        let mut image_paths: Vec<&str> = Vec::new();
        for image in document.images() {
            match image.source() {
                Source::View {
                    view: _,
                    mime_type: _,
                } => {}
                Source::Uri { uri, mime_type: _ } => {
                    image_paths.push(uri);
                    println!("Loading {} {}", image.index(), uri);
                }
            };
        }

        let metadata = Metadata::new(path, &document);

        if document.scenes().len() == 0 {
            return Err(Box::new(ModelLoadingError::new("没有场景")));
        }

        let meshes = create_meshes_from_gltf(&context, command_buffer, &document, &buffers);
        if meshes.is_none() {
            return Err(Box::new(ModelLoadingError::new("没有可渲染的mesh")));
        }

        let Meshes {
            meshes,
            vertices: staged_vertices,
            indices: staged_indices,
        } = meshes.unwrap();

        let scene = document
            .default_scene()
            .unwrap_or_else(|| document.scenes().next().unwrap());

        let animations = load_animations(document.animations(), &buffers);

        let mut skins = create_skins_from_gltf(document.skins(), &buffers);

        let mut node = from_gltf_nodes(document.nodes(), &scene);

        let transform = {
            let aabb = compute_aabb(&node, &meshes);
            let mut transform = compute_unit_cube_at_origin_transform(aabb);
            node.transform(Some(transform.local_to_world_matrix()));
            node.get_skins_transform()
                .iter()
                .for_each(|(index, transform)| {
                    let skin = &mut skins[*index];
                    skin.compute_joints_matrices(*transform, node.nodes());
                });
            transform
        };

        let (textures, staged_textures) = texture::create_textures_from_gltf(
            &context,
            command_buffer,
            document.textures(),
            document.materials(),
            &images,
            image_paths,
        );

        let lights = create_lights_from_gltf(&document);

        let model = Model {
            metadata,
            meshes,
            node,
            transform,
            animations,
            skins,
            textures,
            lights,
        };

        let model_staging_res = ModelStagingResources {
            _staged_vertices: staged_vertices,
            _staged_indices: staged_indices,
            _staged_textures: staged_textures,
        };

        Ok(PreLoadedResource::new(
            context,
            command_buffer,
            model,
            model_staging_res,
        ))
    }
}

impl Model {
    pub fn update(&mut self, delta_time: f32) -> bool {
        let updated = if let Some(animations) = self.animations.as_mut() {
            animations.update(&mut self.nodes, delta_time)
        } else {
            false
        };

        if updated {
            self.nodes
                .transform(Some(self.transform.local_to_world_matrix()));
            self.nodes
                .get_skins_transform()
                .iter()
                .for_each(|(index, transform)| {
                    let skin = &mut self.skins[*index];
                    skin.compute_joints_matrices(*transform, self.nodes.nodes());
                });
        }

        updated
    }
}

impl Model {
    pub fn get_animation_playback_state(&self) -> Option<PlaybackState> {
        self.animations
            .as_ref()
            .map(Animations::get_playback_state)
            .copied()
    }

    pub fn set_current_animation(&mut self, animation_index: usize) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_current(animation_index);
        }
    }

    pub fn set_animation_playback_mode(&mut self, playback_mode: PlaybackMode) {
        if let Some(animations) = self.animations.as_mut() {
            animations.set_playback_mode(playback_mode);
        }
    }

    pub fn toggle_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.toggle();
        }
    }

    pub fn stop_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.stop();
        }
    }

    pub fn reset_animation(&mut self) {
        if let Some(animations) = self.animations.as_mut() {
            animations.reset();
        }
    }

    pub fn update_transform(&mut self) {
        self.node
            .transform(Some(self.transform.local_to_world_matrix()));
    }
}

/// Getters
impl Model {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub fn primitive_count(&self) -> usize {
        self.meshes.iter().map(Mesh::primitive_count).sum()
    }

    pub fn skins(&self) -> &[Skin] {
        &self.skins
    }

    pub fn nodes(&self) -> Rc<Node> {
        self.node.clone()
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures.textures
    }

    pub fn lights(&self) -> &[Light] {
        &self.lights
    }

    pub fn translate(&mut self, position: Vector3<f32>) {
        self.transform.translate(position);
    }

    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        self.transform.rotate(rotation);
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.transform.set_position(position);
    }

    pub fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.transform.set_rotation(rotation);
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.transform.set_scale(scale);
    }
}

fn compute_aabb(nodes: Rc<Node>, meshes: &[Mesh]) -> Aabb<f32> {
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

fn compute_unit_cube_at_origin_transform(aabb: Aabb<f32>) -> Transform {
    let larger_side = aabb.get_larger_side_size();
    let scale_factor = (1.0_f32 / larger_side) * 10.0;

    let aabb = aabb * scale_factor;
    let center = aabb.get_center();

    //let translation = Matrix4::from_translation(-center);
    //let scale = Matrix4::from_scale(scale_factor);
    let transform = Transform::new(
        -center,
        Vector3::<f32>::zero(),
        Vector3::new(scale_factor, scale_factor, scale_factor),
    );
    //translation * scale
    transform
}

pub fn from_gltf_nodes(gltf_nodes: GltfNodes, scene: &Scene) -> Rc<Node> {
    let roots_indices = scene.nodes().map(|n| n.index()).collect::<Vec<_>>();
    let node_count = gltf_nodes.len();
    let mut nodes = Vec::with_capacity(node_count);
    for node in gltf_nodes {
        let node_index = node.index();
        let local_transform = node.transform();
        let global_transform_matrix = compute_transform_matrix(&local_transform);
        let mesh_index = node.mesh().map(|m| m.index());
        let skin_index = node.skin().map(|s| s.index());
        let light_index = node.light().map(|l| l.index());
        let children_indices = node.children().map(|c| c.index()).collect::<Vec<_>>();
        let node = Node {
            local_transform,
            global_transform_matrix,
            mesh_index,
            skin_index,
            light_index,
            children_indices,
        };
        nodes.insert(node_index, node);
    }

    let mut nodes = Nodes::new(nodes, roots_indices);
    nodes.transform(None);
    nodes
}

fn compute_transform_matrix(transform: &Transform) -> Matrix4<f32> {
    match transform {
        Transform::Matrix { matrix } => Matrix4::from(*matrix),
        Transform::Decomposed {
            translation,
            rotation: [xr, yr, zr, wr],
            scale: [xs, ys, zs],
        } => {
            let translation = Matrix4::from_translation(Vector3::from(*translation));
            let rotation = Matrix4::from(Quaternion::new(*wr, *xr, *yr, *zr));
            let scale = Matrix4::from_nonuniform_scale(*xs, *ys, *zs);
            translation * rotation * scale
        }
    }
}
