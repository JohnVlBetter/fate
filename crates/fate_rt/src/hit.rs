use cgmath::{Point3, Vector3};

use crate::ray::Ray;

pub struct HitRecord {
    pub p: Point3<f64>,
    pub normal: Vector3<f64>,
    pub t: f64,
}

pub trait Hit {
    fn hit(&mut self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
