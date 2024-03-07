use image::{GenericImageView, Pixel};

pub const BYTES_PER_PIXEL: usize = 3;

#[derive(Default, Clone)]
pub struct Image {
    image: image::DynamicImage,
    image_width: usize,
    image_height: usize,
    //bytes_per_scanline: usize,
}

impl Image {
    pub fn new(image_filename: &str) -> Self {
        let filename = image_filename;
        let mut _self = Self::default();
        let img = image::open(&format!("res/texture/{}", filename));
        let dyn_img: image::DynamicImage = img.expect("Image loading failed.");
        let (width, height) = (dyn_img.width(), dyn_img.height());
        _self.image = dyn_img;
        _self.image_width = width as usize;
        _self.image_height = height as usize;
        //_self.bytes_per_scanline = BYTES_PER_PIXEL;
        return _self;
    }

    pub fn new_with_dyn_img(dyn_img: image::DynamicImage) -> Self {
        let mut _self = Self::default();
        let (width, height) = (dyn_img.width(), dyn_img.height());
        _self.image = dyn_img;
        _self.image_width = width as usize;
        _self.image_height = height as usize;
        return _self;
    }

    pub fn width(&self) -> usize {
        self.image.width() as usize
    }
    pub fn height(&self) -> usize {
        self.image.height() as usize
    }

    pub fn pixel_data(&self, x: usize, y: usize) -> [u8;3] {
        let x = Self::clamp(x, 0, self.image_width);
        let y = Self::clamp(y, 0, self.image_height);

        let pixel = self.image.get_pixel(x as u32, y as u32);
        pixel.to_rgb().0
    }

    fn clamp(x: usize, low: usize, high: usize) -> usize {
        if x < low {
            return low;
        }
        if x < high {
            return x;
        }
        high - 1
    }
}
