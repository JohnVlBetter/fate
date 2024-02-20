use std::sync::Arc;

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

use crate::aabb::Aabb;
use crate::hit::{Hit, HitRecord};
use crate::interval::Interval;
use crate::material::*;
use crate::ray::Ray;
use crate::utils::random_double;

pub struct Triangle {
    a: Point3<f64>,
    b: Point3<f64>,
    c: Point3<f64>,
    normal: Vector3<f64>,
    mat: Arc<dyn Scatter>,
    bbox: Aabb,
    area: f64,
}

impl Triangle {
    pub fn new(a: Point3<f64>, b: Point3<f64>, c: Point3<f64>, mat: Arc<dyn Scatter>) -> Self {
        let n = (a - c).cross(a - b);
        let normal = n.normalize();
        Self {
            a,
            b,
            c,
            normal,
            mat,
            bbox: Aabb::new_with_points(&a, &b, &c),
            area: normal.magnitude() * 0.5,
        }
    }
}

impl Hit for Triangle {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut u: f64 = 0.0;
        let mut v: f64 = 0.0;
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;
        let s = r.origin - self.a;
        let s1 = Vector3::cross(r.direction, e2);
        let s2 = Vector3::cross(s, e1);
        let coeff = 1.0 / Vector3::dot(s1, e1);
        let t = coeff * Vector3::dot(s2, e2);
        let b1 = coeff * Vector3::dot(s1, s);
        let b2 = coeff * Vector3::dot(s2, r.direction);
        if t >= 0.0 && b1 >= 0.0 && b2 >= 0.0 && (1.0 - b1 - b2) >= 0.0 {
            u = b1;
            v = b2;
        } else {
            return false;
        }
        rec.t = t;
        rec.p = r.at(t);
        rec.normal = self.normal;
        rec.mat = Some(Arc::clone(&self.mat)).unwrap();
        rec.set_face_normal(r, rec.normal);
        return true;
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }
}
