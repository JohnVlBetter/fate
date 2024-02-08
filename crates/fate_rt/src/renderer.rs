use std::{
    io::{stderr, Write},
    ops::Range,
    path::Path,
};

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;

use crate::{camera::Camera, hit::Hit, ray::Ray, scene::World, sphere::Sphere};

const SAMPLES_PER_PIXEL: u64 = 100;
const MAX_DEPTH: u64 = 5;

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, width: usize, height: usize, path: &Path) -> anyhow::Result<()> {
        let mut bytes: Vec<u8> = Vec::with_capacity(width * height * 3);

        // Camera
        let camera = Camera::new(width, height);

        let mut world = World::new();
        world.push(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)?));
        world.push(Box::new(Sphere::new(
            Point3::new(0.0, -100.5, -1.0),
            100.0,
        )?));

        let mut rng = rand::thread_rng();
        for j in (0..height).rev() {
            eprint!("\r进度: {:3}", height - j - 1);
            stderr().flush().unwrap();
            for i in 0..width {
                let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let random_u: f64 = rng.gen();
                    let random_v: f64 = rng.gen();

                    let u = ((i as f64) + random_u) / ((width - 1) as f64);
                    let v = ((j as f64) + random_v) / ((height - 1) as f64);

                    let r = camera.get_ray(u, v);
                    pixel_color += ray_color(&r, &mut world, MAX_DEPTH);
                }
                let final_color = format_color(pixel_color, SAMPLES_PER_PIXEL);

                bytes.push(final_color.x as u8);
                bytes.push(final_color.y as u8);
                bytes.push(final_color.z as u8);
            }
        }

        image::save_buffer(
            path,
            &bytes,
            width as u32,
            height as u32,
            image::ColorType::Rgb8,
        )?;
        eprintln!("渲染完毕");

        Ok(())
    }
}

fn hit_sphere(center: Point3<f64>, radius: f64, r: &Ray) -> f64 {
    let oc = r.origin() - center;
    let a = r.direction().magnitude().powi(2);
    let half_b = oc.dot(r.direction());
    let c = oc.magnitude().powi(2) - radius * radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - discriminant.sqrt()) / a
    }
}

fn ray_color(r: &Ray, world: &mut World, depth: u64) -> Vector3<f64> {
    if depth <= 0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.0, f64::INFINITY) {
        let target = rec.p + rec.normal + random_in_unit_sphere();
        let r = Ray::new(rec.p, target - rec.p);
        0.5 * ray_color(&r, world, depth - 1)
    } else {
        let unit_direction = r.direction().normalize();
        let t = 0.5 * (unit_direction.y + 1.0);
        (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
    }
}

pub fn format_color(color: Vector3<f64>, samples_per_pixel: u64) -> Vector3<u64> {
    let ir = (256.0 * (color[0] / (samples_per_pixel as f64)).sqrt().clamp(0.0, 0.999)) as u64;
    let ig = (256.0 * (color[1] / (samples_per_pixel as f64)).sqrt().clamp(0.0, 0.999)) as u64;
    let ib = (256.0 * (color[2] / (samples_per_pixel as f64)).sqrt().clamp(0.0, 0.999)) as u64;

    Vector3 {
        x: ir,
        y: ig,
        z: ib,
    }
}

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
