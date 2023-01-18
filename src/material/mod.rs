mod lambertian;
mod metal;

pub use metal::Metal;
pub use lambertian::Lambertian;

use crate::{geom::{Ray, Color}, hittable::HitRecord};

pub trait Material {
  fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult;
}

pub struct ScatterResult {
  pub attenuation: Color,
  pub scattered_ray: Option<Ray>
}