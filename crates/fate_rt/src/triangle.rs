use std::sync::Arc;

use cgmath::{InnerSpace, Point3, Vector3};

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
            area: n.magnitude() * 0.5,
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

        true
    }

    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }

    fn pdf_value(&self, origin: Point3<f64>, direction: Vector3<f64>) -> f64 {
        let mut rec = HitRecord {
            p: Point3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            mat: Arc::new(Metal::new(Vector3::new(0.0, 0.0, 0.0), 0.0)),
            t: 0.0,
            u: 0.0,
            v: 0.0,
            front_face: true,
        };
        if !self.hit(
            &Ray::new(origin, direction),
            &Interval::new(0.0001, f64::INFINITY),
            &mut rec,
        ) {
            return 0.0;
        }

        let distance_squared = rec.t * rec.t * direction.magnitude2();
        let cosine = (Vector3::dot(direction, rec.normal) / direction.magnitude()).abs();

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: Point3<f64>) -> Vector3<f64> {
        let mut x = random_double();
        let mut y = random_double();
        if x + y > 1.0 {
            x = 1.0 - x;
            y = 1.0 - y;
        }
        let ab = self.b - self.a;
        let ac = self.c - self.a;
        let p = self.a + x * ab + y * ac;
        return p - origin;
    }
}
