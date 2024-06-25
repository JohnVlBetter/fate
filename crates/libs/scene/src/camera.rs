use glam::{Mat4, Vec3};

use crate::{component::Component, frustum::Frustum};
use std::any::Any;

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    id: u32,
    node_id: u32,
    fov: f32,
    near: f32,
    far: f32,
    aspect: f32,
    target: Vec3,
    position: Vec3,
    is_projection: bool,
}

impl Component for Camera {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        "Camera"
    }

    fn start(&mut self) {}

    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Camera {
    pub fn new(node_id: u32, fov: f32, near: f32, far: f32, aspect: f32) -> Self {
        Camera {
            id: 0,
            node_id,
            fov,
            near,
            far,
            aspect,
            target: Vec3::ZERO,
            position: Vec3::ZERO,
            is_projection: true,
        }
    }

    pub fn set_node_id(&mut self, node_id: u32) {
        self.node_id = node_id;
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn fov(&self) -> f32 {
        self.fov
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
    }

    pub fn near(&self) -> f32 {
        self.near
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
    }

    pub fn far(&self) -> f32 {
        self.far
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    pub fn is_projection(&self) -> bool {
        self.is_projection
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    pub fn get_frustum(&self) -> Frustum {
        Frustum::compute(self.get_projection_matrix(), self.get_view_matrix())
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            id: 0,
            node_id: 0,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
            aspect: 1.0,
            target: Vec3::ONE,
            position: Vec3::ZERO,
            is_projection: true,
        }
    }
}
