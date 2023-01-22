use std::{ops::Range, sync::Arc};

use crate::{geom::Point3, interpolate::lerp, material::Material};

use super::{Hittable, Sphere};

#[derive(Clone)]
pub struct MovingSphere {
    pub center: Range<Point3>,
    pub time: Range<f64>,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl MovingSphere {
    fn center_at(&self, time: f64) -> Point3 {
        let time_portion = (self.time.start - time) / (self.time.end - self.time.start);
        lerp(self.center.start, self.center.end, time_portion)
    }
}

impl Hittable for MovingSphere {
    fn hit(
        &self,
        ray: &crate::geom::Ray,
        t_range: std::ops::Range<f64>,
    ) -> Option<super::HitRecord> {
        let center = self.center_at(ray.time);
        let fixed = Sphere {
            center,
            radius: self.radius,
            material: self.material.clone(),
        };
        fixed.hit(ray, t_range)
    }
}
