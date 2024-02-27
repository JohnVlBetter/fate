use std::{f64::consts::PI, sync::Arc};

use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;

use crate::{
    hit::HitRecord,
    pdf::{CosinePdf, NonePdf, Pdf, SpherePdf},
    ray::Ray,
    texture::{SolidColor, Texture},
    utils::{random_in_unit_sphere, reflect, refract},
};

pub struct ScatterRecord {
    pub attenuation: Vector3<f64>,
    pub pdf: Box<dyn Pdf>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Ray,
}

impl Default for ScatterRecord {
    fn default() -> Self {
        Self {
            attenuation: Vector3::new(0.0, 0.0, 0.0),
            pdf: Box::new(NonePdf {}),
            skip_pdf: false,
            skip_pdf_ray: Ray::new(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0)),
        }
    }
}

pub trait Scatter: Send + Sync {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool;

    fn emitted(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _u: f64,
        _v: f64,
        _p: Point3<f64>,
    ) -> Vector3<f64> {
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo.value(rec.u, rec.v, rec.p);
        srec.pdf = Box::new(CosinePdf::new(rec.normal));
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = Vector3::dot(rec.normal, scattered.direction().normalize());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }
}

pub struct PBR {
    pub albedo: Arc<dyn Texture>,
    //pub ao: Arc<dyn Texture>,
    //pub emissive: Arc<dyn Texture>,
    pub normal: Arc<dyn Texture>,
    //pub metal_roughness: Arc<dyn Texture>,
}

impl PBR {
    pub fn new(albedo: Arc<dyn Texture>, normal: Arc<dyn Texture>) -> Self {
        Self { albedo, normal }
    }
}

impl Scatter for PBR {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo.value(rec.u, rec.v, rec.p);
        srec.pdf = Box::new(CosinePdf::new(rec.normal));
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = Vector3::dot(rec.normal, scattered.direction().normalize());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo;
        srec.skip_pdf = true;
        let reflected = reflect(&r_in.direction().normalize(), &rec.normal);
        srec.skip_pdf_ray = Ray::new(rec.p, reflected + self.fuzz * random_in_unit_sphere());
        true
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = Vector3::new(1.0, 1.0, 1.0);
        srec.skip_pdf = true;

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

        srec.skip_pdf_ray = Ray::new(rec.p, direction);
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
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
        false
    }

    fn emitted(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        u: f64,
        v: f64,
        p: Point3<f64>,
    ) -> Vector3<f64> {
        if rec.front_face {
            self.emit.value(u, v, p)
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        }
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo.value(rec.u, rec.v, rec.p);
        srec.pdf = Box::new(SpherePdf {});
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * PI)
    }
}
