use std::f64::consts::PI;

use cgmath::{InnerSpace, Point3, Vector3};

use crate::{
    hit::Hit, onb::Onb, utils::{random_cosine_direction, random_in_unit_sphere}
};

pub trait Pdf {
    fn value(&self, direction: Vector3<f64>) -> f64;
    fn generate(&self) -> Vector3<f64>;
}

pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _direction: Vector3<f64>) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vector3<f64> {
        random_in_unit_sphere().normalize()
    }
}

pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: Vector3<f64>) -> Self {
        Self {
            uvw: Onb::new_from_w(w),
        }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: Vector3<f64>) -> f64 {
        let cosine_theta = Vector3::dot(direction.normalize(), self.uvw.w());
        0.0_f64.max(cosine_theta / PI)
    }

    fn generate(&self) -> Vector3<f64> {
        self.uvw.local_v(random_cosine_direction())
    }
}

pub struct HittablePdf<'a> {
    pub objects: &'a dyn Hit,
    pub origin: Point3<f64>,
}

impl<'a> HittablePdf<'a> {
    pub fn new(objects: &'a dyn Hit, origin: Point3<f64>) -> Self {
        Self { objects, origin }
    }
}

impl Pdf for HittablePdf<'_> {
    fn value(&self, direction: Vector3<f64>) -> f64 {
        self.objects.pdf_value(self.origin, direction)
    }

    fn generate(&self) -> Vector3<f64> {
        self.objects.random(self.origin)
    }
}
