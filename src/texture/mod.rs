mod solid_color;
mod checkerboard;

pub use solid_color::SolidColor;
pub use checkerboard::Checkerboard;

use crate::geom::{Point3, Color};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

