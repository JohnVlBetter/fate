use std::sync::Arc;

use cgmath::{Point3, Vector3};

use crate::image::Image;

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3<f64>) -> Vector3<f64>;
}

pub struct SolidColor {
    color_value: Vector3<f64>,
}

impl SolidColor {
    pub fn new(color_value: Vector3<f64>) -> Self {
        Self { color_value }
    }

    pub fn new_with_rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            color_value: Vector3::new(r, g, b),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3<f64>) -> Vector3<f64> {
        self.color_value
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    pub fn new_with_color(scale: f64, c1: Vector3<f64>, c2: Vector3<f64>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::new(c1)),
            odd: Arc::new(SolidColor::new(c2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: Point3<f64>) -> Vector3<f64> {
        let x_integer = (self.inv_scale * p.x).floor() as i32;
        let y_integer = (self.inv_scale * p.y).floor() as i32;
        let z_integer = (self.inv_scale * p.z).floor() as i32;

        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

pub struct ImageTexture {
    image: Image,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        Self {
            image: Image::new(filename),
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Point3<f64>) -> Vector3<f64> {
        if self.image.height() == 0 {
            return Vector3::new(0.0, 1.0, 1.0);
        }

        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);

        let i = (u * self.image.width() as f64) as usize;
        let j = (v * self.image.height() as f64) as usize;
        let pixel = self.image.pixel_data(i, j);

        let color_scale = 1.0 / 255.0;
        Vector3::new(
            color_scale * pixel[0] as f64,
            color_scale * pixel[1] as f64,
            color_scale * pixel[2] as f64,
        )
    }
}
