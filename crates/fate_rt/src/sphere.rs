use std::{f64::consts::PI, sync::Arc};

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};

use crate::{
    aabb::Aabb,
    hit::{Hit, HitRecord},
    interval::Interval,
    material::{Metal, Scatter},
    onb::Onb,
    ray::Ray,
    utils::random_double,
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

    fn get_sphere_uv(p: Vector3<f64>) -> (f64, f64) {
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;

        (phi / (2.0 * PI), theta / PI)
    }

    fn random_to_sphere(radius: f64, distance_squared: f64) -> Vector3<f64> {
        let r1 = random_double();
        let r2 = random_double();
        let z = 1.0 + r2 * ((1.0 - radius * radius / distance_squared).sqrt() - 1.0);

        let phi = 2.0 * PI * r1;
        let x = phi.cos() * (1.0 - z * z).sqrt();
        let y = phi.sin() * (1.0 - z * z).sqrt();

        Vector3::new(x, y, z)
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
        (hit_record.u, hit_record.v) = Self::get_sphere_uv(outward_normal);

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
            &Interval::new(0.001, f64::INFINITY),
            &mut rec,
        ) {
            return 0.0;
        }

        let cos_theta_max =
            (1.0 - self.radius * self.radius / (self.center - origin).magnitude2()).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: Point3<f64>) -> Vector3<f64> {
        let direction = self.center - origin;
        let distance_squared = direction.magnitude2();
        let uvw = Onb::new_from_w(direction);
        uvw.local_v(Self::random_to_sphere(self.radius, distance_squared))
    }
}
