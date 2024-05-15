use std::{path::Path, sync::Arc};

use anyhow::Result;
use cgmath::{Point3, Vector3};

use crate::{
    camera::Camera,
    hit::{RotateY, Translate},
    hittable_list::HittableList,
    material::{DiffuseLight, Lambertian, Metal, Scatter},
    model::Model,
    quad::{make_box, Quad},
    texture::ImageTexture,
    transform::Transform,
};

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, _width: usize, _height: usize, path: &Path) -> anyhow::Result<()> {
        cornell_box(path);
        Ok(())
    }
}

fn cornell_box(path: &Path) {
    let mut world = HittableList::default();

    let red: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.73, 0.73, 0.73)));
    let light: Arc<dyn Scatter> =
        Arc::new(DiffuseLight::new_with_color(Vector3::new(50.0, 50.0, 50.0)));

    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 555.0, 0.0),
        Vector3::new(0.0, 0.0, 555.0),
        Arc::clone(&red),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vector3::new(-130.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -105.0),
        Arc::clone(&light),
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

    let metal_mat: Arc<dyn Scatter> = Arc::new(Metal::new(Vector3::new(0.23, 0.23, 0.23), 0.0));
    let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Arc::clone(&metal_mat),
    );
    let box1 = Arc::new(RotateY::new(box1, 30.0));
    let box1 = Arc::new(Translate::new(box1, Vector3::new(320.0, 0.0, 295.0)));
    world.add(box1);

    /*let mut b_transform = Transform::new(
        Vector3::new(100.0, 140.0, 300.0),
        Vector3::new(180.0, 0.0, 0.0),
        Vector3::new(1.0, 1.0, 1.0),
    )
    .unwrap();
    b_transform.update_matrix();
    let bunny = Arc::new(
        Model::new(
            "res/model/Duck/glTF/Duck.gltf",
            1.0,
            b_transform,
        )
        .unwrap(),
    );
    world.add(bunny);*/

    let mut d_transform = Transform::new(
        Vector3::new(200.0, 100.0, 200.0),
        Vector3::new(0.0, 0.0, 180.0),
        Vector3::new(1.0, 1.0, 1.0),
    )
    .unwrap();
    d_transform.update_matrix();
    let dragon = Arc::new(
        Model::new(
            "res/model/FlightHelmet/glTF/FlightHelmet.gltf",
            100.0,
            d_transform,
        )
        .unwrap(),
    );

    let green: Arc<dyn Scatter> = Arc::new(Lambertian::new_with_texture(Arc::new(
        ImageTexture::new("Default_albedo.jpg"),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vector3::new(0.0, 555.0, 0.0),
        Vector3::new(0.0, 0.0, 555.0),
        Arc::clone(&green),
    )));

    world.add(dragon);

    let mut lights = HittableList::default();
    lights.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vector3::new(-130.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -105.0),
        Arc::clone(&light),
    )));

    let mut cam = Camera::default();

    cam.aspect_ratio = 1.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 30;
    cam.background = Vector3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vector3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    cam.render(&world, &lights, path);
}
