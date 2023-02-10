use crate::texture::Texture;

use super::Material;

pub struct DiffuseLight {
    pub texture: Box<dyn Texture>,
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, p: crate::geom::Point3) -> crate::geom::Color {
        self.texture.value(u, v, p)
    }
}