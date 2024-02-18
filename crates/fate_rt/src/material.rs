use std::{f64::consts::PI, sync::Arc};

use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;

use crate::{
    hit::HitRecord,
    onb::Onb,
    ray::Ray,
    texture::{SolidColor, Texture},
    utils::{near_zero, random_cosine_direction, random_in_unit_sphere, reflect, refract},
};

pub trait Scatter: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Vector3<f64>,
        scattered: &mut Ray,
        pdf: &mut f64,
    ) -> bool;

    fn emitted(&self, _u: f64, _v: f64, _p: Point3<f64>) -> Vector3<f64> {
        Vector3::new(0.0, 0.0, 0.0)
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }
}
pub struct Lambertian {
    pub albedo: Arc<dyn Texture>,
}

impl Lambertian {
    pub fn new(color: Vector3<f64>) -> Lambertian {
        Lambertian {
            albedo: Arc::new(SolidColor::new(color)),
        }
    }

    pub fn new_with_texture(tex: Arc<dyn Texture>) -> Self {
        Self { albedo: tex }
    }
}

impl Scatter for Lambertian {
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Vector3<f64>,
        scattered: &mut Ray,
        pdf: &mut f64,
    ) -> bool {
        let uvw = Onb::new_from_w(rec.normal);
        let mut scatter_direction = uvw.local_v(random_cosine_direction());

        if near_zero(&scatter_direction) {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo.value(rec.u, rec.v, rec.p);
        *pdf = Vector3::dot(uvw.w(), scattered.direction()) / PI;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (2.0 * PI)
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
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Vector3<f64>,
        scattered: &mut Ray,
        _pdf: &mut f64,
    ) -> bool {
        let reflected = reflect(&r_in.direction(), &rec.normal).normalize();
        *scattered = Ray::new(rec.p, reflected + self.fuzz * random_in_unit_sphere());

        if scattered.direction().dot(rec.normal) > 0.0 {
            *attenuation = self.albedo;
            true
        } else {
            false
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
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Vector3<f64>,
        scattered: &mut Ray,
        _pdf: &mut f64,
    ) -> bool {
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

        *scattered = Ray::new(rec.p, direction);
        *attenuation = Vector3::new(1.0, 1.0, 1.0);
        true
    }
}

pub struct DiffuseLight {
    pub emit: Arc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(a: Arc<dyn Texture>) -> Self {
        Self { emit: a }
    }

    pub fn new_with_color(c: Vector3<f64>) -> Self {
        Self {
            emit: Arc::new(SolidColor::new(c)),
        }
    }
}

impl Scatter for DiffuseLight {
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Vector3<f64>,
        _scattered: &mut Ray,
        _pdf: &mut f64,
    ) -> bool {
        false
    }

    fn emitted(&self, u: f64, v: f64, p: Point3<f64>) -> Vector3<f64> {
        self.emit.value(u, v, p)
    }
}

pub struct Isotropic {
    pub albedo: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new(a: Arc<dyn Texture>) -> Self {
        Self { albedo: a }
    }

    pub fn new_with_color(c: Vector3<f64>) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(c)),
        }
    }
}

impl Scatter for Isotropic {
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Vector3<f64>,
        scattered: &mut Ray,
        pdf: &mut f64,
    ) -> bool {
        *scattered = Ray::new(rec.p, random_in_unit_sphere().normalize());
        *attenuation = self.albedo.value(rec.u, rec.v, rec.p);
        *pdf = 1.0 / (4.0 * PI);
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
