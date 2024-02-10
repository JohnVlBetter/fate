use std::{path::Path, sync::Arc};

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    camera::Camera,
    hit::Hit,
    material::{Dielectric, Lambertian, Metal},
    ray::Ray,
    scene::World,
    sphere::Sphere,
    utils::random,
};

const SAMPLES_PER_PIXEL: u64 = 5;
const MAX_DEPTH: u64 = 5;

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, width: usize, height: usize, path: &Path) -> anyhow::Result<()> {
        let mut bytes: Vec<u8> = Vec::with_capacity(width * height * 3);

        let world = random_scene();

        let lookfrom = Point3::new(13.0, 2.0, 3.0);
        let lookat = Point3::new(0.0, 0.0, 0.0);
        let vup = Vector3::new(0.0, 1.0, 0.0);
        let dist_to_focus = 10.0;
        let aperture = 0.1;

        let cam = Camera::new(
            lookfrom,
            lookat,
            vup,
            20.0,
            (width / height) as f64,
            aperture,
            dist_to_focus,
        );

        for j in (0..height).rev() {
            eprint!("\r进度: {:3}%", (1.0 - (j as f32 + 1.0) / height as f32) * 100.0);

            let scanline: Vec<Vector3<f64>> = (0..width)
                .into_par_iter()
                .map(|i| {
                    let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                    for _ in 0..SAMPLES_PER_PIXEL {
                        let mut rng = rand::thread_rng();
                        let random_u: f64 = rng.gen();
                        let random_v: f64 = rng.gen();

                        let u = ((i as f64) + random_u) / ((width - 1) as f64);
                        let v = ((j as f64) + random_v) / ((height - 1) as f64);

                        let r = cam.get_ray(u, v);
                        pixel_color += ray_color(&r, &world, MAX_DEPTH);
                    }

                    pixel_color
                })
                .collect();

            for pixel_color in scanline {
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

fn random_scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let ground_mat = Arc::new(Lambertian::new(Vector3::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat).unwrap();

    world.push(Box::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new(
                (a as f64) + rng.gen_range(0.0..0.9),
                0.2,
                (b as f64) + rng.gen_range(0.0..0.9),
            );

            if choose_mat < 0.8 {
                let albedo = random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat).unwrap();

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                let albedo = random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat).unwrap();

                world.push(Box::new(sphere));
            } else {
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat).unwrap();

                world.push(Box::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Vector3::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Vector3::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1).unwrap();
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2).unwrap();
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3).unwrap();

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));

    world
}

fn ray_color(r: &Ray, world: &World, depth: u64) -> Vector3<f64> {
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
