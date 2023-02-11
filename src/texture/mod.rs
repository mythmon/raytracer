mod checkerboard;
mod image;
mod perlin;
mod solid_color;
mod isotropic;

use dyn_clonable::clonable;
use crate::geom::{Color, Point3};

pub use self::image::Image;
pub use checkerboard::Checkerboard;
pub use perlin::Perlin;
pub use solid_color::SolidColor;
pub use isotropic::Isotropic;

#[clonable]
pub trait Texture: Clone + Send + Sync {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}
