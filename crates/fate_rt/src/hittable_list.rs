use std::sync::Arc;

use cgmath::{Point3, Vector3};

use crate::{
    aabb::Aabb,
    hit::{self, Hit, HitRecord},
    interval::Interval,
    material::Metal,
    ray::Ray, utils::random_int,
};

#[derive(Default)]
pub struct HittableList {
    pub objects: Vec<Arc<dyn Hit>>,
    bbox: Aabb,
}

impl HittableList {
    pub fn new(object: Arc<dyn Hit>) -> Self {
        Self {
            objects: vec![object],
            bbox: Aabb::default(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Arc<dyn Hit>) {
        self.bbox = Aabb::new_with_box(&self.bbox, object.bounding_box());
        self.objects.push(object);
    }
}

impl Hit for HittableList {
    fn bounding_box(&self) -> &Aabb {
        &self.bbox
    }

    fn hit(&self, r: &Ray, ray_t: &Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord {
            p: Point3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            mat: Arc::new(Metal::new(Vector3::new(0.0, 0.0, 0.0), 0.0)),
            t: 0.0,
            u: 0.0,
            v: 0.0,
            front_face: true,
        };
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        for object in self.objects.iter() {
            if object.hit(r, &Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }

    fn pdf_value(&self, origin: Point3<f64>, direction: Vector3<f64>) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;

        for object in self.objects.iter() {
            sum += weight * object.pdf_value(origin, direction);
        }

        sum
    }

    fn random(&self, origin: Point3<f64>) -> Vector3<f64> {
        let int_size = self.objects.len() as i32;
        self.objects[random_int(0, int_size - 1) as usize].random(origin)
    }
}
