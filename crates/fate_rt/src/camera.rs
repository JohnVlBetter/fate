use std::{path::Path, sync::Arc};

use cgmath::{InnerSpace, Point3, Vector3};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    hit::{Hit, HitRecord},
    interval::Interval,
    material::{Metal, ScatterRecord},
    pdf::{HittablePdf, MixturePdf, Pdf},
    ray::Ray,
    utils::{degrees_to_radians, random_double, random_in_unit_disk},
};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: usize,
    pub samples_per_pixel: usize,
    pub max_depth: usize,
    pub background: Vector3<f64>,
    pub vfov: f64,
    pub lookfrom: Point3<f64>,
    pub lookat: Point3<f64>,
    pub vup: Vector3<f64>,
    pub defocus_angle: f64,
    pub focus_dist: f64,
    image_height: usize,
    sqrt_spp: usize,
    recip_sqrt_spp: f64,
    center: Point3<f64>,
    pixel00_loc: Point3<f64>,
    pixel_delta_u: Vector3<f64>,
    pixel_delta_v: Vector3<f64>,
    u: Vector3<f64>,
    v: Vector3<f64>,
    w: Vector3<f64>,
    defocus_disk_u: Vector3<f64>,
    defocus_disk_v: Vector3<f64>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            background: Vector3::new(0.0, 0.0, 0.0),
            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, -1.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            vup: Vector3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
            image_height: 0,
            sqrt_spp: 10.0_f64.sqrt() as usize,
            recip_sqrt_spp: 1.0 / (10.0_f64.sqrt()),
            center: Point3::new(0.0, 0.0, 0.0),
            pixel00_loc: Point3::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vector3::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vector3::new(0.0, 0.0, 0.0),
            u: Vector3::new(0.0, 0.0, 0.0),
            v: Vector3::new(0.0, 0.0, 0.0),
            w: Vector3::new(0.0, 0.0, 0.0),
            defocus_disk_u: Vector3::new(0.0, 0.0, 0.0),
            defocus_disk_v: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Camera {
    pub fn render(&mut self, world: &dyn Hit, lights: &dyn Hit, path: &Path) {
        self.initialize();

        let mut bytes: Vec<u8> = Vec::with_capacity(self.image_width * self.image_height * 3);

        for j in 0..self.image_height {
            eprint!(
                "\r进度: {:3}%",
                (1.0 - (j as f32 + 1.0) / self.image_height as f32) * 100.0
            );

            let scanline: Vec<Vector3<f64>> = (0..self.image_width)
                .into_par_iter()
                .map(|i| {
                    let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                    for s_j in 0..self.sqrt_spp {
                        for s_i in 0..self.sqrt_spp {
                            let r = self.get_ray(i as i32, j as i32, s_i as i32, s_j as i32);
                            pixel_color += self.ray_color(&r, self.max_depth, world, lights);
                        }
                    }

                    pixel_color
                })
                .collect();

            for pixel_color in scanline {
                let final_color = format_color(pixel_color, self.samples_per_pixel);

                bytes.push(final_color.x as u8);
                bytes.push(final_color.y as u8);
                bytes.push(final_color.z as u8);
            }
        }

        let _ = image::save_buffer(
            path,
            &bytes,
            self.image_width as u32,
            self.image_height as u32,
            image::ColorType::Rgb8,
        );
        eprintln!("渲染完毕");
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as usize;
        self.image_height = if self.image_height < 1 {
            1
        } else {
            self.image_height
        };
        self.sqrt_spp = (self.samples_per_pixel as f64).sqrt() as usize;
        self.recip_sqrt_spp = 1.0 / (self.sqrt_spp as f64);

        self.center = self.lookfrom;

        let theta = degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.w = (self.lookfrom - self.lookat).normalize();
        self.u = Vector3::cross(self.vup, self.w).normalize();
        self.v = Vector3::cross(self.w, self.u);

        let viewport_u = self.u * viewport_width;
        let viewport_v = -self.v * viewport_height;

        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        let viewport_upper_left =
            self.center - (self.focus_dist * self.w) - (0.5 * viewport_u) - (0.5 * viewport_v);
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        let defocus_radius = self.focus_dist * (degrees_to_radians(self.defocus_angle / 2.0)).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    fn get_ray(&self, i: i32, j: i32, s_i: i32, s_j: i32) -> Ray {
        let pixel_center =
            self.pixel00_loc + i as f64 * self.pixel_delta_u + j as f64 * self.pixel_delta_v;
        let pixel_sample = pixel_center + self.pixel_sample_square(s_i, s_j);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn pixel_sample_square(&self, s_i: i32, s_j: i32) -> Vector3<f64> {
        let px = -0.5 + self.recip_sqrt_spp * (s_i as f64 + random_double());
        let py = -0.5 + self.recip_sqrt_spp * (s_j as f64 + random_double());
        px * self.pixel_delta_u + py * self.pixel_delta_v
    }

    fn defocus_disk_sample(&self) -> Point3<f64> {
        let p = random_in_unit_disk();
        self.center + p.x * self.defocus_disk_u + p.y * self.defocus_disk_v
    }

    fn ray_color(&self, r: &Ray, depth: usize, world: &dyn Hit, lights: &dyn Hit) -> Vector3<f64> {
        let mut rec = HitRecord {
            p: Point3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            mat: Arc::new(Metal::new(Vector3::new(0.0, 0.0, 0.0), 0.0)),
            t: 0.0,
            u: 0.0,
            v: 0.0,
            front_face: true,
        };

        if depth <= 0 {
            return Vector3::new(0.0, 0.0, 0.0);
        }

        if !world.hit(r, &Interval::new(0.001, f64::INFINITY), &mut rec) {
            return self.background;
        }

        let mut srec = ScatterRecord::default();
        let color_from_emission = rec.mat.emitted(r, &rec, rec.u, rec.v, rec.p);
        if !rec.mat.scatter(r, &rec, &mut srec) {
            return color_from_emission;
        }

        if srec.skip_pdf {
            let skip_pdf_ray_color = self.ray_color(&srec.skip_pdf_ray, depth - 1, world, lights);
            return Vector3::new(
                srec.attenuation.x * skip_pdf_ray_color.x,
                srec.attenuation.y * skip_pdf_ray_color.y,
                srec.attenuation.z * skip_pdf_ray_color.z,
            );
        }

        let light_pdf = HittablePdf::new(lights, rec.p);
        let mixed_pdf = MixturePdf::new(&light_pdf, &*srec.pdf);

        let scattered = Ray::new(rec.p, mixed_pdf.generate());
        let pdf = mixed_pdf.value(scattered.direction());

        let scattering_pdf = rec.mat.scattering_pdf(r, &rec, &scattered);
        let col = self.ray_color(&scattered, depth - 1, world, lights);
        let color_from_scatter = Vector3::new(
            srec.attenuation.x * col.x * scattering_pdf,
            srec.attenuation.y * col.y * scattering_pdf,
            srec.attenuation.z * col.z * scattering_pdf,
        ) / pdf;

        color_from_emission + color_from_scatter
    }
}

pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}
const INTENSITY: Interval = Interval {
    min: 0.0,
    max: 0.999,
};

pub fn format_color(color: Vector3<f64>, samples_per_pixel: usize) -> Vector3<u64> {
    let r = color.x;
    let g = color.y;
    let b = color.z;

    let r = if r.is_nan() { 0.0 } else { r };
    let g = if g.is_nan() { 0.0 } else { g };
    let b = if b.is_nan() { 0.0 } else { b };

    let scale = 1.0 / samples_per_pixel as f64;
    let r = scale * r;
    let g = scale * g;
    let b = scale * b;

    let r = linear_to_gamma(r);
    let g = linear_to_gamma(g);
    let b = linear_to_gamma(b);

    Vector3 {
        x: (256.0 * INTENSITY.clamp(r)) as u64,
        y: (256.0 * INTENSITY.clamp(g)) as u64,
        z: (256.0 * INTENSITY.clamp(b)) as u64,
    }
}
