use anyhow::Result;
use cgmath::{Deg, SquareMatrix};

use crate::mesh::{Mat4, Vec3};

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
        * Mat4::from_angle_x(Deg(90.0))
        * Mat4::from_angle_y(Deg(90.0))
        * Mat4::from_angle_z(Deg(90.0))
        * Mat4::from_scale(10.0);
        self.local_to_world_matrix
    }
}
