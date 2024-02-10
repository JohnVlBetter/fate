use cgmath::{InnerSpace, Point3, Vector3};

use crate::{ray::Ray, utils::random_in_unit_disk};

pub struct Camera {
    origin: Point3<f64>,
    lower_left_corner: Point3<f64>,
    horizontal: Vector3<f64>,
    vertical: Vector3<f64>,
    cu: Vector3<f64>,
    cv: Vector3<f64>,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3<f64>,
        lookat: Point3<f64>,
        vup: Vector3<f64>,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Camera {
        let theta = std::f64::consts::PI / 180.0 * vfov;
        let viewport_height = 2.0 * (theta / 2.0).tan();
        let viewport_width = aspect_ratio * viewport_height;

        let cw = (lookfrom - lookat).normalize();
        let cu = vup.cross(cw).normalize();
        let cv = cw.cross(cu);

        let h = focus_dist * viewport_width * cu;
        let v = focus_dist * viewport_height * cv;

        let llc = lookfrom - h / 2.0 - v / 2.0 - focus_dist * cw;

        Camera {
            origin: lookfrom,
            horizontal: h,
            vertical: v,
            lower_left_corner: llc,
            cu: cu,
            cv: cv,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.cu * rd.x + self.cv * rd.y;

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}
