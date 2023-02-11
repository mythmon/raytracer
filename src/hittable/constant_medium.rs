use std::{ops::Range, sync::Arc};
use rand::Rng;
use crate::{
    geom::{Aabb, Vec3},
    texture::{Isotropic, Texture},
};
use super::{HitRecord, Hittable};

#[derive(Clone)]
pub struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Arc<Isotropic>,
}

impl ConstantMedium {
    pub fn new(boundary: Box<dyn Hittable>, density: f64, texture: Box<dyn Texture>) -> Self {
        Self {
            boundary,
            neg_inv_density: -density.recip(),
            phase_function: Arc::new(Isotropic { albedo: texture }),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: crate::geom::Ray, t_range: Range<f64>) -> Option<super::HitRecord> {
        let mut rng = rand::thread_rng();
        let enable_debug = false;
        let debug = enable_debug && rng.gen_bool(0.00001);

        let mut rec1 = self.boundary.hit(ray, -f64::INFINITY..f64::INFINITY)?;
        let mut rec2 = self.boundary.hit(ray, (rec1.t + 0.0001)..f64::INFINITY)?;

        if debug { dbg!(&rec1.t, &rec2.t); }

        if rec1.t < t_range.start {
            rec1.t = t_range.start;
        }
        if rec2.t > t_range.end {
            rec2.t = t_range.end;
        }

        if rec1.t >= rec2.t {
            return None;
        }

        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_length = ray.direction.length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * rng.gen::<f64>().log10();

        if debug { dbg!(&hit_distance, &distance_inside_boundary); }

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = rec1.t + hit_distance / ray_length;
        let p = ray.along(t);

        if debug { dbg!(&t, &p); }

        Some(HitRecord::new(
            p,
            t,
            ray,
            Vec3::new(1.0, 0.0, 0.0),
            self.phase_function.clone(),
            0.0,
            0.0,
        ))
    }

    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb> {
        self.boundary.bounding_box(time_range)
    }
}
