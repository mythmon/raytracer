use crate::geom::{Color, Point3};

use super::Texture;

pub struct SolidColor(pub Color);

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.0
    }
}