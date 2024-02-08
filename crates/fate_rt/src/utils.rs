use std::ops::Range;

use cgmath::{InnerSpace, Vector3};
use rand::Rng;

pub fn random(r: Range<f64>) -> Vector3<f64> {
    let mut rng = rand::thread_rng();

    Vector3 {
        x: rng.gen_range(r.clone()),
        y: rng.gen_range(r.clone()),
        z: rng.gen_range(r.clone()),
    }
}

pub fn random_in_unit_sphere() -> Vector3<f64> {
    loop {
        let v = random(-1.0..1.0);
        if v.magnitude() < 1.0 {
            return v;
        }
    }
}

pub fn random_in_hemisphere(normal: Vector3<f64>) -> Vector3<f64> {
    let in_unit_sphere = random_in_unit_sphere();
    if in_unit_sphere.dot(normal) > 0.0 {
        in_unit_sphere
    } else {
        (-1.0) * in_unit_sphere
    }
}

pub fn near_zero(vec: &Vector3<f64>) -> bool {
    const EPS: f64 = 1.0e-8;
    vec[0].abs() < EPS && vec[1].abs() < EPS && vec[2].abs() < EPS
}

pub fn reflect(vec: &Vector3<f64>, n: &Vector3<f64>) -> Vector3<f64> {
    vec - 2.0 * vec.dot(*n) * n
}

pub fn refract(vec: &Vector3<f64>, n: &Vector3<f64>, etai_over_etat: f64) -> Vector3<f64> {
    let cos_theta = ((-1.0) * vec).dot(*n).min(1.0);
    let r_out_perp = etai_over_etat * (vec + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.magnitude().powi(2)).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}