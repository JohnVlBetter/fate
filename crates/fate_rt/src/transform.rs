use anyhow::Result;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3, Vector4};

use crate::{hit::HitRecord, ray::Ray};

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f64>,
    pub euler: Vector3<f64>,
    pub scale: Vector3<f64>,

    local_to_world_matrix: Matrix4<f64>,
    world_to_local_matrix: Matrix4<f64>,
}

impl Transform {
    pub fn new(position: Vector3<f64>, euler: Vector3<f64>, scale: Vector3<f64>) -> Result<Self> {
        Ok(Self {
            position,
            euler,
            scale,
            local_to_world_matrix: Matrix4::identity(),
            world_to_local_matrix: Matrix4::identity(),
        })
    }

    pub fn update_matrix(&mut self) {
        self.local_to_world_matrix = Matrix4::from_translation(self.position)
            * Matrix4::from_angle_x(cgmath::Rad(self.euler.x))
            * Matrix4::from_angle_y(cgmath::Rad(self.euler.y))
            * Matrix4::from_angle_z(cgmath::Rad(self.euler.z))
            * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.world_to_local_matrix = self.local_to_world_matrix.invert().unwrap();
    }

    pub fn transform_ray(&self, r: &Ray) -> Ray {
        let origin = Vector4::new(r.origin.x, r.origin.y, r.origin.z, 1.0);
        let origin = self.world_to_local_matrix * origin;
        let origin = Point3::new(origin.x, origin.y, origin.z);
        let direction = Vector4::new(r.direction.x, r.direction.y, r.direction.z, 0.0);
        let direction = self.world_to_local_matrix * direction;
        let direction = Vector3::new(direction.x, direction.y, direction.z);
        let new_ray = Ray::new(origin, direction);
        new_ray
    }

    pub fn transform_rec(&self, rec: &mut HitRecord) {
        let point = Vector4::new(rec.p.x, rec.p.y, rec.p.z, 1.0);
        let point = self.local_to_world_matrix * point;
        rec.p.x = point.x;
        rec.p.y = point.y;
        rec.p.z = point.z;
        let normal = Vector4::new(rec.normal.x, rec.normal.y, rec.normal.z, 0.0);
        let normal = self.local_to_world_matrix * normal;
        rec.normal.x = normal.x;
        rec.normal.y = normal.y;
        rec.normal.z = normal.z;
    }
}
