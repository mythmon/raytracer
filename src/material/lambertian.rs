use crate::{
    geom::{Ray, Vec3},
    hittable::HitRecord,
    material::{Material, ScatterResult},
    texture::Texture,
};

pub struct Lambertian {
    pub albedo: Box<dyn Texture>,
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult {
        let mut scatter_direction = hit_record.normal + Vec3::rand_unit_vector();

        // avoid degenerate scatter directions that can cause divide by zeros later
        if scatter_direction.is_near_zero() {
            scatter_direction = hit_record.normal;
        }

        ScatterResult {
            attenuation: self.albedo.value(hit_record.u, hit_record.v, hit_record.p),
            scattered_ray: Some(Ray::new(hit_record.p, scatter_direction, ray_in.time)),
        }
    }
}
