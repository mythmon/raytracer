mod checkerboard;
mod image;
mod perlin;
mod solid_color;

pub use self::image::Image;
pub use checkerboard::Checkerboard;
pub use perlin::Perlin;
pub use solid_color::SolidColor;

use crate::geom::{Color, Point3};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}
