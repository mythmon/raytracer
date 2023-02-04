mod solid_color;
mod checkerboard;
mod perlin;

pub use solid_color::SolidColor;
pub use checkerboard::Checkerboard;
pub use perlin::Perlin;

use crate::geom::{Point3, Color};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

