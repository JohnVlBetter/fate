use std::sync::Arc;

use cgmath::{InnerSpace, Point3, Vector2, Vector3};

use crate::aabb::Aabb;
use crate::hit::{Hit, HitRecord};
use crate::image::Image;
use crate::interval::Interval;
use crate::material::*;
use crate::ray::Ray;
use crate::utils::random_double;
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub(crate) pos: Point3<f64>,
    pub(crate) color: Vector3<f64>,
    pub(crate) normal: Vector3<f64>,
    pub(crate) tex_coord: Vector2<f64>,
}

impl Vertex {
    pub fn new(
        pos: Point3<f64>,
        color: Vector3<f64>,
        normal: Vector3<f64>,
        tex_coord: Vector2<f64>,
    ) -> Self {
        Self {
            pos,
            color,
            normal,
            tex_coord,
        }
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
            && self.color == other.color
            && self.normal == other.normal
            && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

pub struct Triangle {
    a: Vertex,
    b: Vertex,
    c: Vertex,
    mat: Arc<dyn Scatter>,
    _normal_texture: Arc<Image>,
    bbox: Aabb,
    area: f64,
}

impl Triangle {
    pub fn new(
        a: Vertex,
        b: Vertex,
        c: Vertex,
        mat: Arc<dyn Scatter>,
        _normal_texture: Arc<Image>,
    ) -> Self {
        Self {
            a,
            b,
            c,
            mat,
            _normal_texture,
            bbox: Aabb::new_with_points(&a.pos, &b.pos, &c.pos),
            area: (a.pos - c.pos).cross(a.pos - b.pos).magnitude() * 0.5,
        }
    }
}

impl Hit for Triangle {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let e1 = self.b.pos - self.a.pos;
        let e2 = self.c.pos - self.a.pos;
        let p = r.direction.cross(e2);
        let det = e1.dot(p);

        if det > -::std::f64::EPSILON && det < ::std::f64::EPSILON {
            return false;
        }

        let inv_det = 1.0 / det;
        let s = r.origin - self.a.pos;
        let beta = inv_det * s.dot(p);
        if beta < 0.0 || beta > 1.0 {
            return false;
        }

        let q = s.cross(e1);
        let gamma = inv_det * r.direction.dot(q);
        if gamma < 0.0 || beta + gamma > 1.0 {
            return false;
        }

        let t = inv_det * e2.dot(q);

        if t < ray_t.min || t > ray_t.max {
            return false;
        } else {
            let intersection_point = r.at(t);

            let alpha = 1.0 - beta - gamma;

            let normal = self.a.normal * alpha + self.b.normal * beta + self.c.normal * gamma;

            let u = self.a.tex_coord[0] * alpha
                + self.b.tex_coord[0] * beta
                + self.c.tex_coord[0] * gamma;
            let v = self.a.tex_coord[1] * alpha
                + self.b.tex_coord[1] * beta
                + self.c.tex_coord[1] * gamma;

            rec.t = t;
            rec.p = intersection_point;
            rec.u = u;
            rec.v = v;
            rec.normal = normal;
            rec.mat = Arc::clone(&self.mat);
            rec.set_face_normal(r, rec.normal);
        }

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
        let mut x = random_double();
        let mut y = random_double();
        if x + y > 1.0 {
            x = 1.0 - x;
            y = 1.0 - y;
        }
        let ab = self.b.pos - self.a.pos;
        let ac = self.c.pos - self.a.pos;
        let p = self.a.pos + x * ab + y * ac;
        return p - origin;
    }
}
