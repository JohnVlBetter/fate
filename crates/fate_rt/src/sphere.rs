use std::sync::Arc;

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};

use crate::{
    aabb::Aabb,
    hit::{Hit, HitRecord},
    interval::Interval,
    material::Scatter,
    ray::Ray,
};

pub struct Sphere {
    pub center: Point3<f64>,
    pub radius: f64,
    pub mat: Arc<dyn Scatter>,
    pub bbox: Aabb,
}

impl Sphere {
    pub fn new(center: Point3<f64>, radius: f64, mat: Arc<dyn Scatter>) -> Result<Self> {
        let rvec = Vector3::new(radius, radius, radius);
        Ok(Self {
            center,
            radius,
            mat,
            bbox: Aabb::new_with_point(&(center - rvec), &(center + rvec)),
        })
    }
}

impl Hit for Sphere {
    fn hit(&self, ray: &Ray, ray_t: &Interval, hit_record: &mut HitRecord) -> bool {
        let oc = ray.origin() - self.center;
        let a = ray.direction().magnitude().powi(2);
        let half_b = oc.dot(ray.direction());
        let c = oc.magnitude().powi(2) - self.radius.powi(2);

        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (-half_b + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        let p: Point3<f64> = ray.at(root);
        hit_record.t = root;
        hit_record.p = p;
        hit_record.mat = self.mat.clone();

        let outward_normal = (hit_record.p - self.center) / self.radius;
        hit_record.set_face_normal(&ray, outward_normal);

        true
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}
