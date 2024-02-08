use cgmath::{InnerSpace, Vector3};

use crate::{hit::HitRecord, ray::Ray, utils::{near_zero, random_in_unit_sphere, reflect}};

pub trait Scatter {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)>;
}
pub struct Lambertian {
    albedo: Vector3<f64>,
}

impl Lambertian {
    pub fn new(a: Vector3<f64>) -> Lambertian {
        Lambertian { albedo: a }
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let mut scatter_direction = rec.normal + random_in_unit_sphere().normalize();
        if near_zero(&scatter_direction) {
            scatter_direction = rec.normal;
        }

        let scattered = Ray::new(rec.p, scatter_direction);

        Some((self.albedo, scattered))
    }
}

pub struct Metal {
    albedo: Vector3<f64>
}

impl Metal {
    pub fn new(a: Vector3<f64>) -> Metal {
        Metal {
            albedo: a
        }
    }
}

impl Scatter for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let reflected = reflect(&r_in.direction(), &rec.normal).normalize();
        let scattered = Ray::new(rec.p, reflected);

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}