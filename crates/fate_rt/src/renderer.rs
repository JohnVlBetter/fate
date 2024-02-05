use std::{io::{stderr, Write}, path::Path};

use anyhow::Result;

#[derive(Copy, Clone, Debug)]
pub struct Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn render(&self, width: usize, height: usize, path: &Path) -> anyhow::Result<()> {
        let mut bytes: Vec<u8> = Vec::with_capacity(width * height * 3);

        for j in (0..height).rev() {
            eprint!("\r进度: {:3}", height - j - 1);
            stderr().flush().unwrap();
            for i in 0..width {
                let r = (i as f64) / ((width - 1) as f64);
                let g = (j as f64) / ((height - 1) as f64);
                let b = 0.25;

                let ir = (255.999 * r) as u64;
                let ig = (255.999 * g) as u64;
                let ib = (255.999 * b) as u64;

                bytes.push(ir as u8);
                bytes.push(ig as u8);
                bytes.push(ib as u8);
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
