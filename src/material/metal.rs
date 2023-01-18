use crate::{
    geom::{Color, Ray, Vec3},
    material::Material,
};

pub struct Metal {
    pub albedo: Color,
    pub fuzziness: f64,
}

impl Material for Metal {
    fn scatter(
        &self,
        ray_in: &crate::geom::Ray,
        hit_record: &crate::hittable::HitRecord,
    ) -> super::ScatterResult {
        let reflected = ray_in.direction.unit_vector().reflect(hit_record.normal);
        let scattered_ray = if reflected.dot(hit_record.normal) > 0.0 {
            let direction = reflected + self.fuzziness * Vec3::rand_unit_vector();
            Some(Ray::new(hit_record.p, direction))
        } else {
            None
        };
        super::ScatterResult {
            attenuation: self.albedo,
            scattered_ray,
        }
    }
}
