use crate::geom::Axis;
use crate::texture::Texture;

pub struct Checkerboard {
    even: Box<dyn Texture>,
    odd: Box<dyn Texture>,
}

impl Checkerboard {
    pub fn new(even: Box<dyn Texture>, odd: Box<dyn Texture>) -> Self {
        Self { even, odd }
    }
}

impl Texture for Checkerboard {
    fn value(&self, u: f64, v: f64, p: crate::geom::Point3) -> crate::geom::Color {
        let sines: f64 = [Axis::X, Axis::Y, Axis::Z]
            .into_iter()
            .map(|a| (p[a] * 10.0).sin())
            .product();

        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
