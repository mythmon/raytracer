use anyhow::Result;
use image::RgbImage;
use std::{fs::File, io::BufReader, path::Path};
use super::Texture;

#[derive(Clone)]
pub struct Image {
    image: RgbImage,
}

impl Image {
    pub fn new(path: &Path) -> Result<Self> {
        let f = File::open(path)?;
        let f = BufReader::new(f);
        let f = image::io::Reader::new(f).with_guessed_format()?;
        let image = f.decode()?;
        Ok(Self {
            image: image.into_rgb8(),
        })
    }
}

impl Texture for Image {
    fn value(&self, u: f64, v: f64, _p: crate::geom::Point3) -> crate::geom::Color {
        // let u = u.clamp(0.0, 1.0);
        // let v = 1.0 - v.clamp(0.0, 1.0);

        let w = self.image.width();
        let h = self.image.height();
        let i = ((u * w as f64) as u32).clamp(0, w - 1);
        let j = (h - (v * h as f64) as u32).clamp(0, h - 1);

        let p = self.image.get_pixel(i, j);
        p.into()
    }
}
