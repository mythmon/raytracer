mod dielectric;
mod lambertian;
mod metal;

pub use dielectric::Dielectric;
pub use lambertian::Lambertian;
pub use metal::Metal;

use crate::{
    geom::{Color, Ray},
    hittable::HitRecord,
};

pub trait Material: Send + Sync {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult;
}

pub struct ScatterResult {
    pub attenuation: Color,
    pub scattered_ray: Option<Ray>,
}
