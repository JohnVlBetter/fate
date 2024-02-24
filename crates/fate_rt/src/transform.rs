use anyhow::Result;
use cgmath::{vec3, Deg, Matrix4, SquareMatrix, Vector3};

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f64>,
    pub euler: Vector3<f64>,
    pub scale: Vector3<f64>,

    local_to_world_matrix: Matrix4<f64>,
}

impl Transform {
    pub fn new(position: Vector3<f64>, euler: Vector3<f64>, scale: Vector3<f64>) -> Result<Self> {
        Ok(Self {
            position,
            euler,
            scale,
            local_to_world_matrix: Matrix4::identity(),
        })
    }

    pub fn local_to_world_matrix(&mut self) -> Matrix4<f64> {
        self.local_to_world_matrix = Matrix4::from_translation(self.position)
            * Matrix4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(-120.0))
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.local_to_world_matrix
    }
}
