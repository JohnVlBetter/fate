use std::{path::Path, sync::Arc};

use anyhow::Result;
use cgmath::{Point3, Vector3};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    bvh::BvhNode,
    camera::Camera,
    constant_medium::ConstantMedium,
    hit::{Hit, HitRecord, RotateY, Translate},
    hittable_list::HittableList,
    interval::Interval,
    material::{Dielectric, DiffuseLight, Lambertian, Metal, Scatter, ScatterRecord},
    pdf::{HittablePdf, MixturePdf, Pdf},
    quad::{make_box, Quad},
    ray::Ray,
    sphere::Sphere,
    texture::{CheckerTexture, ImageTexture, Texture},
    utils::{random, random_double_range},
};

const SAMPLES_PER_PIXEL: u64 = 100;
const MAX_DEPTH: u64 = 15;

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, width: usize, height: usize, path: &Path) -> anyhow::Result<()> {
        let mut bytes: Vec<u8> = Vec::with_capacity(width * height * 3);

        let mut world = cornell_box();
        let world = HittableList::new(Arc::new(BvhNode::new(&mut world)));

        // Light Sources.
        let light: Arc<dyn Scatter> =
            Arc::new(DiffuseLight::new_with_color(Vector3::new(15.0, 15.0, 15.0)));
        let mut lights = HittableList::default();
        lights.add(Arc::new(Quad::new(
            Point3::new(343.0, 554.0, 332.0),
            Vector3::new(-130.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, -105.0),
            Arc::clone(&light),
        )));

        let lookfrom = Point3::new(278.0, 278.0, -800.0);
        let lookat = Point3::new(278.0, 278.0, 0.0);
        let vup = Vector3::new(0.0, 1.0, 0.0);
        let dist_to_focus = 10.0;
        let aperture = 0.1;

        let cam = Camera::new(
            lookfrom,
            lookat,
            vup,
            40.0,
            width as f64 / height as f64,
            aperture,
            dist_to_focus,
        );

        for j in (0..height).rev() {
            eprint!(
                "\r进度: {:3}%",
                (1.0 - (j as f32 + 1.0) / height as f32) * 100.0
            );

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
                        pixel_color +=
                            ray_color(&r, &world, &lights, MAX_DEPTH, Vector3::new(0.0, 0.0, 0.0));
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

fn earth() -> HittableList {
    let earth_texture: Arc<dyn Texture> = Arc::new(ImageTexture::new("earthmap.jpg"));
    let earth_surface = Arc::new(Lambertian::new_with_texture(Arc::clone(&earth_texture)));
    let globe = Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, earth_surface).unwrap();
    let mut world = HittableList::default();
    world.add(Arc::new(globe));
    world
}

fn quads() -> HittableList {
    let mut world = HittableList::default();

    // Material
    let left_red = Arc::new(Lambertian::new(Vector3::new(1.0, 0.2, 0.2)));
    let back_green = Arc::new(Lambertian::new(Vector3::new(0.2, 1.0, 0.2)));
    let right_blue = Arc::new(Lambertian::new(Vector3::new(0.2, 0.2, 1.0)));
    let upper_orange = Arc::new(Lambertian::new(Vector3::new(1.0, 0.5, 0.0)));
    let lower_teal = Arc::new(Lambertian::new(Vector3::new(0.2, 0.8, 0.8)));

    // Quad
    world.add(Arc::new(Quad::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vector3::new(0.0, 0.0, -4.0),
        Vector3::new(0.0, 4.0, 0.0),
        left_red,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vector3::new(4.0, 0.0, 0.0),
        Vector3::new(0.0, 4.0, 0.0),
        back_green,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(3.0, -2.0, 1.0),
        Vector3::new(0.0, 0.0, 4.0),
        Vector3::new(0.0, 4.0, 0.0),
        right_blue,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vector3::new(4.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 4.0),
        upper_orange,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vector3::new(4.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -4.0),
        lower_teal,
    )));

    world
}

fn random_scene() -> HittableList {
    let mut rng = rand::thread_rng();
    let mut world = HittableList::default();

    let checker: Arc<dyn Texture> = Arc::new(CheckerTexture::new_with_color(
        0.32,
        Vector3::new(0.2, 0.3, 0.1),
        Vector3::new(0.9, 0.9, 0.9),
    ));
    let ground_material = Arc::new(Lambertian::new_with_texture(Arc::clone(&checker)));
    let ground_sphere =
        Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_material).unwrap();

    world.add(Arc::new(ground_sphere));

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

                world.add(Arc::new(sphere));
            } else if choose_mat < 0.95 {
                let albedo = random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat).unwrap();

                world.add(Arc::new(sphere));
            } else {
                let sphere_mat = Arc::new(Dielectric::new(1.5));
                let sphere = Sphere::new(center, 0.2, sphere_mat).unwrap();

                world.add(Arc::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5));
    let mat2 = Arc::new(Lambertian::new(Vector3::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Vector3::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1).unwrap();
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2).unwrap();
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3).unwrap();

    world.add(Arc::new(sphere1));
    world.add(Arc::new(sphere2));
    world.add(Arc::new(sphere3));

    world
}

fn simple_light() -> HittableList {
    let mut world = HittableList::default();

    world.add(Arc::new(
        Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            Arc::new(Lambertian::new(Vector3::new(0.7, 0.6, 0.2))),
        )
        .unwrap(),
    ));
    world.add(Arc::new(
        Sphere::new(
            Point3::new(0.0, 2.0, 0.0),
            2.0,
            Arc::new(Lambertian::new(Vector3::new(0.4, 0.2, 0.7))),
        )
        .unwrap(),
    ));

    let difflight = Arc::new(DiffuseLight::new_with_color(Vector3::new(4.0, 4.0, 4.0)));
    world.add(Arc::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vector3::new(2.0, 0.0, 0.0),
        Vector3::new(0.0, 2.0, 0.0),
        difflight,
    )));

    world
}

fn cornell_box() -> HittableList {
    let mut world = HittableList::default();

    let red = Arc::new(Lambertian::new(Vector3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Vector3::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new_with_color(Vector3::new(15.0, 15.0, 15.0)));

    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vector3::new(0.0, 555.0, 0.0),
        Vector3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 555.0, 0.0),
        Vector3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vector3::new(-130.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -105.0),
        light,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(555.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 555.0, 555.0),
        Vector3::new(-555.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vector3::new(555.0, 0.0, 0.0),
        Vector3::new(0.0, 555.0, 0.0),
        Arc::clone(&white),
    )));

    let aluminum: Arc<dyn Scatter> = Arc::new(Metal::new(Vector3::new(0.8, 0.85, 0.88), 0.0));
    let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Arc::clone(&aluminum),
    );
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vector3::new(265.0, 0.0, 295.0)));
    world.add(box1);

    let box2 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    );
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vector3::new(130.0, 0.0, 65.0)));
    world.add(box2);

    world
}

fn final_scene() -> HittableList {
    let mut boxes1 = HittableList::default();
    let ground: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    (0..boxes_per_side).for_each(|i| {
        (0..boxes_per_side).for_each(|j| {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_double_range(1.0, 101.0);
            let z1 = z0 + w;

            boxes1.add(make_box(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                Arc::clone(&ground),
            ));
        });
    });

    let mut world = HittableList::default();

    world.add(Arc::new(BvhNode::new(&mut boxes1)));

    let light: Arc<dyn Scatter> =
        Arc::new(DiffuseLight::new_with_color(Vector3::new(7.0, 7.0, 7.0)));
    world.add(Arc::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vector3::new(412.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 412.0),
        light,
    )));

    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vector3::new(30.0, 0.0, 0.0);
    let sphere_scatter: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.7, 0.3, 0.1)));
    world.add(Arc::new(
        Sphere::new(center1, 50.0, sphere_scatter).unwrap(),
    ));

    world.add(Arc::new(
        Sphere::new(
            Point3::new(260.0, 150.0, 45.0),
            50.0,
            Arc::new(Dielectric::new(1.5)),
        )
        .unwrap(),
    ));
    world.add(Arc::new(
        Sphere::new(
            Point3::new(0.0, 150.0, 145.0),
            50.0,
            Arc::new(Metal::new(Vector3::new(0.8, 0.8, 0.9), 1.0)),
        )
        .unwrap(),
    ));

    let boundary: Arc<dyn Hit> = Arc::new(
        Sphere::new(
            Point3::new(360.0, 150.0, 145.0),
            70.0,
            Arc::new(Dielectric::new(1.5)),
        )
        .unwrap(),
    );
    world.add(Arc::clone(&boundary));
    world.add(Arc::new(ConstantMedium::new_with_Vector3(
        Arc::clone(&boundary),
        0.2,
        Vector3::new(0.2, 0.4, 0.9),
    )));
    let boundary: Arc<dyn Hit> = Arc::new(
        Sphere::new(
            Point3::new(0.0, 0.0, 0.0),
            5000.0,
            Arc::new(Dielectric::new(1.5)),
        )
        .unwrap(),
    );
    world.add(Arc::new(ConstantMedium::new_with_Vector3(
        Arc::clone(&boundary),
        0.0001,
        Vector3::new(1.0, 1.0, 1.0),
    )));

    let emat: Arc<dyn Scatter> = Arc::new(Lambertian::new_with_texture(Arc::new(
        ImageTexture::new("earthmap.jpg"),
    )));
    world.add(Arc::new(
        Sphere::new(Point3::new(400.0, 200.0, 400.0), 100.0, emat).unwrap(),
    ));
    let pertext = Arc::new(Lambertian::new(Vector3::new(0.2, 0.4, 0.6)));
    world.add(Arc::new(
        Sphere::new(Point3::new(220.0, 280.0, 300.0), 80.0, pertext).unwrap(),
    ));

    let mut boxes2 = HittableList::default();
    let white: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    let mut rng = rand::thread_rng();
    (0..ns).for_each(|_| {
        boxes2.add(Arc::new(
            Sphere::new(
                Point3 {
                    x: rng.gen_range(0.0..165.0),
                    y: rng.gen_range(0.0..165.0),
                    z: rng.gen_range(0.0..165.0),
                },
                10.0,
                Arc::clone(&white),
            )
            .unwrap(),
        ));
    });

    world.add(Arc::new(Translate::new(
        Arc::new(RotateY::new(Arc::new(BvhNode::new(&mut boxes2)), 15.0)),
        Vector3::new(-100.0, 270.0, 395.0),
    )));

    world
}

fn ray_color(
    r: &Ray,
    world: &dyn Hit,
    lights: &dyn Hit,
    depth: u64,
    background: Vector3<f64>,
) -> Vector3<f64> {
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
        return background;
    }

    let mut srec = ScatterRecord::default();
    let color_from_emission = rec.mat.emitted(r, &rec, rec.u, rec.v, rec.p);
    if !rec.mat.scatter(r, &rec, &mut srec) {
        return color_from_emission;
    }

    if srec.skip_pdf {
        let skip_pdf_ray_color =
            ray_color(&srec.skip_pdf_ray, world, lights, depth - 1, background);
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
    let col = ray_color(&scattered, world, lights, depth - 1, background);
    let color_from_scatter = Vector3::new(
        srec.attenuation.x * col.x * scattering_pdf,
        srec.attenuation.y * col.y * scattering_pdf,
        srec.attenuation.z * col.z * scattering_pdf,
    ) / pdf;

    color_from_emission + color_from_scatter
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
