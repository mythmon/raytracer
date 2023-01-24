use crate::{
    geom::{Point3, Ray, Aabb, Vec3},
    hittable::{HitRecord, Hittable},
    material::Material,
};
use std::{f64::consts::PI, ops::Range, sync::Arc};

#[derive(Clone)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    /// Given a point `p` on a sphere of radius 1 centered at the origin,
    /// calculates:
    /// - `u` a value in `0..=1`, the angle around the Y axis, from X=-1 to X=+1
    /// - `v` a value in `0..=1`, the angle around the X axis, rfom Y=-1 to Y=+1
    fn get_uv(p: Point3) -> (f64, f64) {
        let theta = p.y().acos();
        let phi = f64::atan2(-p.z(), p.x()) + PI;
        (phi / 2.0 * PI, theta / PI)
    }
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
            let (u, v) = Self::get_uv(outward_normal);
            Some(HitRecord::new(
                p,
                t,
                &ray,
                outward_normal,
                self.material.clone(),
                u,
                v,
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
