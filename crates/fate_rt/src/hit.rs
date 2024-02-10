use std::sync::Arc;

use cgmath::{InnerSpace, Point3, Vector3};

use crate::{material::Scatter, ray::Ray};

pub struct HitRecord {
    pub p: Point3<f64>,
    pub normal: Vector3<f64>,
    pub mat: Arc<dyn Scatter>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vector3<f64>) -> () {
        self.front_face = r.direction().dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            (-1.0) * outward_normal
        };
    }
}

pub trait Hit: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
