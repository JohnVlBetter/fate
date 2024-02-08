use std::{
    io::{stderr, Write},
    path::Path, rc::Rc,
};

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;

use crate::{camera::Camera, hit::Hit, material::{Lambertian, Metal}, ray::Ray, scene::World, sphere::Sphere};

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
        let mat_ground = Rc::new(Lambertian::new(Vector3::new(0.8, 0.8, 0.0)));
        let mat_center = Rc::new(Lambertian::new(Vector3::new(0.7, 0.3, 0.3)));
        let mat_left = Rc::new(Metal::new(Vector3::new(0.8, 0.8, 0.8)));
        let mat_right = Rc::new(Metal::new(Vector3::new(0.8, 0.6, 0.2)));

        let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground)?;
        let sphere_center = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_center)?;
        let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left)?;
        let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right)?;

        world.push(Box::new(sphere_ground));
        world.push(Box::new(sphere_center));
        world.push(Box::new(sphere_left));
        world.push(Box::new(sphere_right));

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

fn ray_color(r: &Ray, world: &mut World, depth: u64) -> Vector3<f64> {
    if depth <= 0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            let mut ray_col = ray_color(&scattered, world, depth - 1);
            ray_col.x *= attenuation.x;
            ray_col.y *= attenuation.y;
            ray_col.z *= attenuation.z;
            ray_col
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalize();
        let t = 0.5 * (unit_direction.y + 1.0);
        (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
    }
}

pub fn format_color(color: Vector3<f64>, samples_per_pixel: u64) -> Vector3<u64> {
    let ir = (256.0
        * (color[0] / (samples_per_pixel as f64))
            .sqrt()
            .clamp(0.0, 0.999)) as u64;
    let ig = (256.0
        * (color[1] / (samples_per_pixel as f64))
            .sqrt()
            .clamp(0.0, 0.999)) as u64;
    let ib = (256.0
        * (color[2] / (samples_per_pixel as f64))
            .sqrt()
            .clamp(0.0, 0.999)) as u64;

    Vector3 {
        x: ir,
        y: ig,
        z: ib,
    }
}
