mod lambertian;
mod metal;
mod dielectric;

pub use metal::Metal;
pub use lambertian::Lambertian;
pub use dielectric::Dielectric;

use crate::{geom::{Ray, Color}, hittable::HitRecord};

pub trait Material {
  fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult;
}

pub struct ScatterResult {
  pub attenuation: Color,
  pub scattered_ray: Option<Ray>
}