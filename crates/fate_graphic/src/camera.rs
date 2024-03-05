use anyhow::Result;
use cgmath::{Deg, Matrix4, Point3};

use crate::mesh::Vec3;
type Mat4 = cgmath::Matrix4<f32>;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    pub eye: Point3<f32>,
    pub center: Point3<f32>,
    pub up: Vec3,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub unsafe fn new(
        eye: Point3<f32>,
        center: Point3<f32>,
        up: Vec3,
        fovy: f32,
        near: f32,
        far: f32,
    ) -> Result<Self> {
        Ok(Self {
            eye,
            center,
            up,
            fovy,
            near,
            far,
        })
    }

    pub unsafe fn get_view_mat(&self) -> Matrix4<f32> {
        Mat4::look_at_rh(self.eye, self.center, self.up)
    }

    pub unsafe fn get_proj_mat(&self, width: f32, height: f32) -> Matrix4<f32> {
        cgmath::perspective(Deg(self.fovy), width / height, self.near, self.far)
    }
}
