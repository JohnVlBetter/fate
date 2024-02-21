use std::{path::Path, sync::Arc};

use anyhow::Result;
use cgmath::{Point3, Vector2, Vector3};

use crate::{
    camera::Camera,
    hit::{RotateY, Translate},
    hittable_list::HittableList,
    material::{Dielectric, DiffuseLight, Lambertian, Metal, Scatter},
    model::Model,
    quad::{make_box, Quad},
    sphere::Sphere,
    triangle::{Triangle, Vertex},
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
    let green: Arc<dyn Scatter> = Arc::new(Lambertian::new(Vector3::new(0.12, 0.45, 0.15)));
    let light: Arc<dyn Scatter> =
        Arc::new(DiffuseLight::new_with_color(Vector3::new(50.0, 50.0, 50.0)));

    /*world.add(Arc::new(Quad::new(
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
    )));*/
    world.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vector3::new(-130.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -105.0),
        Arc::clone(&light),
    )));
    /*world.add(Arc::new(Quad::new(
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
    )));*/

    let metal_mat: Arc<dyn Scatter> = Arc::new(Metal::new(Vector3::new(0.23, 0.23, 0.23), 0.0));
    /*let box1 = make_box(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Arc::clone(&metal_mat),
    );
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vector3::new(265.0, 0.0, 295.0)));
    world.add(box1);*/

    /*let glass: Arc<dyn Scatter> = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(
        Sphere::new(Point3::new(190.0, 90.0, 190.0), 90.0, Arc::clone(&glass)).unwrap(),
    ));*/
    let v1 = Vertex::new(
        Point3::new(0.0, 300.0, 10.0),
        Vector3::new(50.0, 200.0, 190.0),
        Vector3::new(300.0, 200.0, 190.0),
        Vector2::new(0.0, 0.0),
    );
    let v2 = Vertex::new(
        Point3::new(50.0, 300.0, 190.0),
        Vector3::new(190.0, 90.0, 190.0),
        Vector3::new(300.0, 200.0, 190.0),
        Vector2::new(0.0, 0.0),
    );
    let v3 = Vertex::new(
        Point3::new(300.0, 300.0, 190.0),
        Vector3::new(190.0, 90.0, 190.0),
        Vector3::new(50.0, 200.0, 190.0),
        Vector2::new(0.0, 0.0),
    );
    world.add(Arc::new(
        Triangle::new(v1, v2, v3, Arc::clone(&light))
    ));
    world.add(Arc::new(
        Model::new("res/model/viking_room/viking_room.obj", Arc::clone(&white)).unwrap(),
    ));

    // Light SouArces.
    let mut lights = HittableList::default();
    lights.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vector3::new(-130.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, -105.0),
        Arc::clone(&light),
    )));
    lights.add(Arc::new(
        Triangle::new(v1, v2, v3, Arc::clone(&light))
    ));

    let mut cam = Camera::default();

    cam.aspect_ratio = 1.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 20;
    cam.background = Vector3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vector3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    cam.render(&world, &lights, path);
}
