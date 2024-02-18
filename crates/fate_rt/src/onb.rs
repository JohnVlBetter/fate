use std::ops::{Index, IndexMut};

use cgmath::{InnerSpace, Vector3};

pub struct Onb {
    pub axis: [Vector3<f64>; 3],
}

impl Onb {
    pub fn u(&self) -> Vector3<f64> {
        self.axis[0]
    }
    pub fn v(&self) -> Vector3<f64> {
        self.axis[1]
    }
    pub fn w(&self) -> Vector3<f64> {
        self.axis[2]
    }

    pub fn local(&self, a: f64, b: f64, c: f64) -> Vector3<f64> {
        a * self.u() + b * self.v() + c * self.w()
    }
    pub fn local_v(&self, a: Vector3<f64>) -> Vector3<f64> {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }

    pub fn new_from_w(w: Vector3<f64>) -> Self {
        let unit_w = w.normalize();
        let a = if unit_w.x.abs() > 0.9 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let v = Vector3::cross(unit_w, a).normalize();
        let u = Vector3::cross(unit_w, v);
        Self {
            axis: [u, v, unit_w],
        }
    }
}

impl Index<usize> for Onb {
    type Output = Vector3<f64>;

    fn index(&self, i: usize) -> &Self::Output {
        &self.axis[i]
    }
}

impl IndexMut<usize> for Onb {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.axis[i]
    }
}
