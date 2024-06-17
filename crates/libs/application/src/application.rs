use std::sync::{Arc, Mutex};

use scene::{
    component::{Camera, Component, ComponentBase, Light, MeshRenderer, Transform},
    scene::Scene,
};

pub struct Application {
    pub(crate) scene: Scene,
}

impl Application {
    fn get_instance() -> Arc<Mutex<Application>> {
        static mut APPLICATION: Option<Arc<Mutex<Application>>> = None;

        unsafe {
            APPLICATION
                .get_or_insert_with(|| {
                    Arc::new(Mutex::new(Self {
                        ..Default::default()
                    }))
                })
                .clone()
        }
    }
    pub fn new() -> Application {
        Application::default()
    }
    pub fn update(&mut self) {
        self.scene.update();
    }
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}

pub fn add_transform_component(node_id: u32) {
    let mut transform = Transform {
        id: 0,
        node_id,
        matrix: "Matrix".to_string(),
    };
    transform.start();
    let transform = Component::Transform(transform);
    Application::get_instance()
        .lock()
        .unwrap()
        .scene
        .scene_tree
        .add_component(node_id, transform);
}

pub fn add_camera_component(node_id: u32) {
    let mut camera = Camera {
        id: 0,
        node_id,
        view: "View".to_string(),
    };
    camera.start();
    let camera = Component::Camera(camera);
    Application::get_instance()
        .lock()
        .unwrap()
        .scene
        .scene_tree
        .add_component(node_id, camera);
}

pub fn add_light_component(node_id: u32) {
    let mut light = Light {
        id: 0,
        node_id,
        color: "Color".to_string(),
    };
    light.start();
    let light = Component::Light(light);
    Application::get_instance()
        .lock()
        .unwrap()
        .scene
        .scene_tree
        .add_component(node_id, light);
}

pub fn add_mesh_renderer_component(node_id: u32) {
    let mut mesh_renderer = MeshRenderer {
        id: 0,
        node_id,
        mesh: "Mesh".to_string(),
    };
    mesh_renderer.start();
    let mesh_renderer = Component::MeshRenderer(mesh_renderer);
    Application::get_instance()
        .lock()
        .unwrap()
        .scene
        .scene_tree
        .add_component(node_id, mesh_renderer);
}
