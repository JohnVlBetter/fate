use anyhow::Result;
use cgmath::{vec3, Deg, SquareMatrix};

use crate::model::{Mat4, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub euler: Vec3,
    pub scale: Vec3,

    local_to_world_matrix: Mat4,
}

impl Transform {
    pub fn new(position: Vec3, euler: Vec3, scale: Vec3) -> Result<Self> {
        Ok(Self {
            position,
            euler,
            scale,
            local_to_world_matrix: Mat4::identity(),
        })
    }

    pub fn local_to_world_matrix(&mut self) -> Mat4 {
        self.local_to_world_matrix = Mat4::from_translation(self.position)
            * Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(-120.0))
            * Mat4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.local_to_world_matrix
    }
}