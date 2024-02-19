use std::f64::consts::PI;

use cgmath::{InnerSpace, Point3, Vector3};

use crate::{
    hit::Hit,
    onb::Onb,
    utils::{random_cosine_direction, random_double_range, random_in_unit_sphere},
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

pub struct MixturePdf<'a> {
    pub p: [&'a dyn Pdf; 2],
}

impl<'a> MixturePdf<'a> {
    pub fn new(p0: &'a dyn Pdf, p1: &'a dyn Pdf) -> Self {
        Self { p: [p0, p1] }
    }
}

impl Pdf for MixturePdf<'_> {
    fn value(&self, direction: Vector3<f64>) -> f64 {
        0.5 * self.p[0].value(direction) + 0.5 * self.p[1].value(direction)
    }

    fn generate(&self) -> Vector3<f64> {
        if random_double_range(0.0, 1.0) < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}

pub struct NonePdf;

impl Pdf for NonePdf {
    fn value(&self, _direction: Vector3<f64>) -> f64 {
        0.0
    }
    fn generate(&self) -> Vector3<f64> {
        Vector3::new(1.0, 0.0, 0.0)
    }
}
