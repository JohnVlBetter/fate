use std::any::Any;
use std::sync::Arc;
use std::{fs::File, io::BufReader, path::Path};

use resource::resource::Resource;
use resource::resource_loader::ResourceLoader;

use image::{codecs::hdr::HdrDecoder, Rgb};

#[derive(Debug, Clone)]
pub struct HDRTextureSource {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
}

impl Resource for HDRTextureSource {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Default)]
pub struct HdrTextureLoader;

impl ResourceLoader for HdrTextureLoader {
    fn load(&self, path: &str) -> Option<Arc<dyn Resource>> {
        let (width, height, data) = load_hdr_image(path);
        Some(Arc::new(HDRTextureSource {
            width,
            height,
            data,
        }))
    }

    fn extensions(&self) -> &[&str] {
        &["hdr"]
    }
}

fn load_hdr_image<P: AsRef<Path>>(path: P) -> (u32, u32, Vec<f32>) {
    let decoder = HdrDecoder::new(BufReader::new(File::open(path).unwrap())).unwrap();
    let w = decoder.metadata().width;
    let h = decoder.metadata().height;
    let rgb = decoder.read_image_hdr().unwrap();
    let mut data = Vec::with_capacity(rgb.len() * 4);
    for Rgb(p) in rgb.iter() {
        data.extend_from_slice(p);
        data.push(0.0);
    }
    (w, h, data)
}
