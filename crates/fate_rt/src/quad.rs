use std::sync::Arc;

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3};

use crate::aabb::Aabb;
use crate::hit::{Hit, HitRecord};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::*;
use crate::ray::Ray;
use crate::utils::random_double;

pub struct Quad {
    q: Point3<f64>,
    u: Vector3<f64>,
    v: Vector3<f64>,
    w: Vector3<f64>,
    normal: Vector3<f64>,
    d: f64,
    mat: Arc<dyn Scatter>,
    bbox: Aabb,
    area: f64,
}

impl Quad {
    pub fn new(q: Point3<f64>, u: Vector3<f64>, v: Vector3<f64>, mat: Arc<dyn Scatter>) -> Self {
        let n = Vector3::cross(u, v);
        let normal = Vector3::normalize(n);
        Self {
            q,
            u,
            v,
            w: n / Vector3::dot(n, n),
            normal,
            d: Vector3::dot(normal, q.to_vec()),
            mat,
            bbox: Aabb::new_with_point(&q, &(q + u + v)),
            area: n.magnitude(),
        }
    }

    pub fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) {
            return false;
        }

        rec.u = a;
        rec.v = b;

        true
    }
}

impl Hit for Quad {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let denom = Vector3::dot(self.normal, r.direction());

        if denom.abs() < 1e-8 {
            return false;
        }

        let t = (self.d - Vector3::dot(self.normal, r.origin().to_vec())) / denom;
        if !ray_t.contains(t) {
            return false;
        }

        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = Vector3::dot(self.w, Vector3::cross(planar_hitpt_vector, self.v));
        let beta = Vector3::dot(self.w, Vector3::cross(self.u, planar_hitpt_vector));

        if !self.is_interior(alpha, beta, rec) {
            return false;
        }

        rec.t = t;
        rec.p = intersection;
        rec.mat = Some(Arc::clone(&self.mat)).unwrap();
        rec.set_face_normal(r, self.normal);

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
        let p = self.q + (random_double() * self.u) + (random_double() * self.v);
        return p - origin;
    }
}

pub fn make_box(a: Point3<f64>, b: Point3<f64>, mat: Arc<dyn Scatter>) -> Arc<HittableList> {
    let mut sides = HittableList::default();

    let min = Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    let dx = Vector3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vector3::new(0.0, max.y - min.y, 0.0);
    let dz = Vector3::new(0.0, 0.0, max.z - min.z);

    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, max.z),
        dx,
        dy,
        Arc::clone(&mat),
    )));
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, max.z),
        -dz,
        dy,
        Arc::clone(&mat),
    )));
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, min.z),
        -dx,
        dy,
        Arc::clone(&mat),
    )));
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dz,
        dy,
        Arc::clone(&mat),
    )));
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, max.y, max.z),
        dx,
        -dz,
        Arc::clone(&mat),
    )));
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dx,
        dz,
        Arc::clone(&mat),
    )));

    Arc::new(sides)
}
