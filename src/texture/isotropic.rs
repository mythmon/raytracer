use crate::{material::Material, geom::{Ray, Vec3}};

use super::Texture;

#[derive(Clone)]
pub struct Isotropic {
    pub albedo: Box<dyn Texture>
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &crate::geom::Ray, hit_record: &crate::hittable::HitRecord) -> crate::material::ScatterResult {
        crate::material::ScatterResult {
            attenuation: self.albedo.value(hit_record.u, hit_record.v, hit_record.p),
            scattered_ray: Some(Ray::new(hit_record.p, Vec3::rand_unit_vector(), ray_in.time)),
        }
    }
}