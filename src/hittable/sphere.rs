use crate::{
    geom::{Point3, Ray, Aabb, Vec3},
    hittable::{HitRecord, Hittable},
    material::Material,
};
use std::{ops::Range, sync::Arc};

#[derive(Clone)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord> {
        // this is based off solving the equation for a sphere set equal to the
        // equation of a line, which boils down to a quadratic equation.

        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius.powi(2);

        let discrim = half_b.powi(2) - a * c;
        if discrim < 0.0 {
            None
        } else {
            // find the nearest root that lies in `t_range`
            let sqrtd = discrim.sqrt();

            let mut root = (-half_b - sqrtd) / a;
            if !t_range.contains(&root) {
                root = (-half_b + sqrtd) / a;
                if !t_range.contains(&root) {
                    return None;
                }
            }
            let t = root;
            let p = ray.along(t);
            let outward_normal = (p - self.center) / self.radius;
            Some(HitRecord::new(
                p,
                t,
                &ray,
                outward_normal,
                self.material.clone(),
            ))
        }
    }

    fn bounding_box(&self, _time_range: Range<f64>) -> Option<Aabb> {
        Some(Aabb::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius),
        ))
    }
}
