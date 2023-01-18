use crate::{material::{Material, ScatterResult}, geom::{Color, Ray, Vec3}, hittable::HitRecord};

pub struct Lambertian {
  pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult {
        let mut scatter_direction = hit_record.normal + Vec3::rand_unit_vector();

        // avoid degenerate scatter directions that can cause divide by zeros later
        if scatter_direction.is_near_zero() {
            scatter_direction = hit_record.normal
        }

        ScatterResult {
            attenuation: self.albedo,
            scattered_ray: Some(Ray::new(hit_record.p, scatter_direction)),
        }
    }
}