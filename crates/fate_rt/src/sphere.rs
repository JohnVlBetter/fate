use anyhow::Result;
use cgmath::{InnerSpace, Point3};

use crate::{
    hit::{Hit, HitRecord},
    ray::Ray,
};

pub struct Sphere {
    pub center: Point3<f64>,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Point3<f64>, radius: f64) -> Result<Self> {
        Ok(Self { center, radius })
    }
}

impl Hit for Sphere {
    fn hit(&mut self, ray: Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin() - self.center;
        let a = ray.direction().magnitude().powi(2);
        let half_b = oc.dot(ray.direction());
        let c = oc.magnitude().powi(2) - self.radius.powi(2);
        
        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0.0 {
            return None;
        }
    
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || t_max < root {
            root = (-half_b + sqrtd) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
    
        let p = ray.at(root);
        let rec = HitRecord {
            t: root,
            p: p,
            normal: (p - self.center) / self.radius
        };
    
        Some(rec)
    }
}