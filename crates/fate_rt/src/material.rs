use cgmath::{InnerSpace, Vector3};
use rand::Rng;

use crate::{
    hit::HitRecord,
    ray::Ray,
    utils::{near_zero, random_in_unit_sphere, reflect, refract},
};

pub trait Scatter: Send + Sync {
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let mut scatter_direction = rec.normal + random_in_unit_sphere().normalize();
        if near_zero(&scatter_direction) {
            scatter_direction = rec.normal;
        }

        let scattered = Ray::new(rec.p, scatter_direction);

        Some((self.albedo, scattered))
    }
}

pub struct Metal {
    albedo: Vector3<f64>,
    fuzz: f64,
}

impl Metal {
    pub fn new(a: Vector3<f64>, f: f64) -> Metal {
        Metal { albedo: a, fuzz: f }
    }
}

impl Scatter for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let reflected = reflect(&r_in.direction(), &rec.normal).normalize();
        let scattered = Ray::new(rec.p, reflected + self.fuzz * random_in_unit_sphere());

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    ir: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64) -> Dielectric {
        Dielectric {
            ir: index_of_refraction,
        }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Scatter for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Vector3<f64>, Ray)> {
        let refraction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().normalize();
        let cos_theta = ((-1.0) * unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);

        Some((Vector3::new(1.0, 1.0, 1.0), scattered))
    }
}
