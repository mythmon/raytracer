use rand::Rng;

use crate::{
    geom::{Color, Ray},
    hittable::HitRecord,
    material::{Material, ScatterResult},
};

pub struct Dielectric {
    pub index_of_refraction: f64,
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterResult {
        let refraction_ratio = if hit_record.front_face {
            self.index_of_refraction.recip()
        } else {
            self.index_of_refraction
        };

        let unit_direction = ray_in.direction.unit_vector();
        let cos_theta = (-unit_direction).dot(hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let mut rng = rand::thread_rng();
        let should_reflect = self.reflectance(cos_theta, refraction_ratio) > rng.gen::<f64>();
        let direction = if cannot_refract || should_reflect {
            unit_direction.reflect(hit_record.normal)
        } else {
            unit_direction.refract(hit_record.normal, refraction_ratio)
        };

        ScatterResult {
            attenuation: Color::white(),
            scattered_ray: Some(Ray::new(hit_record.p, direction, ray_in.time)),
        }
    }
}

impl Dielectric {
    fn reflectance(&self, cosine: f64, ref_idx: f64) -> f64 {
        // Schlick's approximation
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}
