use std::{
    io::{stderr, Write},
    path::Path,
};

use anyhow::Result;
use cgmath::{InnerSpace, Point3, Vector3};

use crate::ray::Ray;

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, width: usize, height: usize, path: &Path) -> anyhow::Result<()> {
        let mut bytes: Vec<u8> = Vec::with_capacity(width * height * 3);

        // Camera
        let viewport_height = 2.0;
        let viewport_width = (width / height) as f64 * viewport_height;
        let focal_length = 1.0;

        let origin = Point3::new(0.0, 0.0, 0.0);
        let horizontal = Vector3::new(viewport_width, 0.0, 0.0);
        let vertical = Vector3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - horizontal / 2.0 - vertical / 2.0 - Vector3::new(0.0, 0.0, focal_length);

        for j in (0..height).rev() {
            eprint!("\r进度: {:3}", height - j - 1);
            stderr().flush().unwrap();
            for i in 0..width {
                let u = (i as f64) / ((width - 1) as f64);
                let v = (j as f64) / ((height - 1) as f64);

                let r = Ray::new(
                    origin,
                    lower_left_corner + u * horizontal + v * vertical - origin,
                );
                let pixel_color = ray_color(&r);
                let final_color = format_color(pixel_color);

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

fn ray_color(r: &Ray) -> Vector3<f64> {
    let t = hit_sphere(
        Point3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        0.5,
        r,
    );
    if t > 0.0 {
        let normal = (r.at(t)
            - Point3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            })
        .normalize();
        return Vector3::new(1.0 + normal.x, 1.0 + normal.y, 1.0 + normal.z) * 0.5;
    }

    let unit_direction = r.direction().normalize();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0)
}

pub fn format_color(color: Vector3<f64>) -> Vector3<f64> {
    Vector3 {
        x: (255.999 * color[0]) as f64,
        y: (255.999 * color[1]) as f64,
        z: (255.999 * color[2]) as f64,
    }
}
