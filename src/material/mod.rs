mod dielectric;
mod lambertian;
mod metal;
mod emissive;

pub use dielectric::Dielectric;
pub use lambertian::Lambertian;
pub use metal::Metal;
pub use emissive::DiffuseLight;

use crate::{
    geom::{Color, Ray, Point3},
    hittable::HitRecord,
};

pub trait Material: Send + Sync {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> ScatterResult {
        ScatterResult { attenuation: Color::black(), scattered_ray: None }
    }

    fn emitted(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        Color::black()
    }
}

pub struct ScatterResult {
    pub attenuation: Color,
    pub scattered_ray: Option<Ray>,
}
