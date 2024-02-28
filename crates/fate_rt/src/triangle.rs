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
    normal: Vector3<f64>,
    mat: Arc<dyn Scatter>,
    normal_texture: Arc<Image>,
    bbox: Aabb,
    area: f64,
}

impl Triangle {
    pub fn new(
        a: Vertex,
        b: Vertex,
        c: Vertex,
        mat: Arc<dyn Scatter>,
        normal_texture: Arc<Image>,
    ) -> Self {
        let n = (a.pos - c.pos).cross(a.pos - b.pos);
        let normal = n.normalize();
        Self {
            a,
            b,
            c,
            normal,
            mat,
            normal_texture,
            bbox: Aabb::new_with_points(&a.pos, &b.pos, &c.pos),
            area: n.magnitude() * 0.5,
        }
    }
}

impl Hit for Triangle {
    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut u: f64 = 0.0;
        let mut v: f64 = 0.0;
        let e1 = self.b.pos - self.a.pos;
        let e2 = self.c.pos - self.a.pos;
        let s = r.origin - self.a.pos;
        let s1 = Vector3::cross(r.direction, e2);
        let s2 = Vector3::cross(s, e1);
        let coeff = 1.0 / Vector3::dot(s1, e1);
        let t = coeff * Vector3::dot(s2, e2);
        let b1 = coeff * Vector3::dot(s1, s);
        let b2 = coeff * Vector3::dot(s2, r.direction);
        if t >= 0.0 && b1 >= 0.0 && b2 >= 0.0 && (1.0 - b1 - b2) >= 0.0 {
            u = b1;
            v = b2;
        } else {
            return false;
        }
        rec.t = t;
        rec.p = r.at(t);

        let f1 = self.a.pos - rec.p;
        let f2 = self.b.pos - rec.p;
        let f3 = self.c.pos - rec.p;
        let a = Vector3::cross(self.a.pos - self.b.pos, self.a.pos - self.c.pos).magnitude(); // main triangle area a
        let a1 = Vector3::cross(f2, f3).magnitude() / a;
        let a2 = Vector3::cross(f3, f1).magnitude() / a;
        let a3 = Vector3::cross(f1, f2).magnitude() / a;
        rec.u = a1 * self.a.tex_coord.x + a2 * self.b.tex_coord.x + a3 * self.c.tex_coord.x;
        rec.v = a1 * self.a.tex_coord.y + a2 * self.b.tex_coord.y + a3 * self.c.tex_coord.y;

        let i = (u * self.normal_texture.width() as f64) as usize;
        let j = (v * self.normal_texture.height() as f64) as usize;
        let pixel = self.normal_texture.pixel_data(i, j);
        rec.normal = Vector3::new(
            pixel[0] as f64 * 2.0 - 1.0,
            pixel[1] as f64 * 2.0 - 1.0,
            pixel[2] as f64 * 2.0 - 1.0,
        );
        rec.normal = self.normal;
        rec.mat = Some(Arc::clone(&self.mat)).unwrap();
        rec.set_face_normal(r, rec.normal);

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
