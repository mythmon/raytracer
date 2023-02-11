use std::{ops::Range, sync::Arc};

use crate::{
    geom::{Aabb, Axis, Point3, Ray, Vec3},
    material::Material,
};

use super::{HitRecord, Hittable};

#[derive(Clone)]
pub struct AxisAlignedRect {
    pub center: Point3,
    pub width: f64,
    pub height: f64,
    pub axis: Axis,
    pub material: Arc<dyn Material>,
}

impl AxisAlignedRect {
    fn bounding_box(&self) -> Aabb {
        // The bounding box must have non-zero width in each dimension, so pad
        // the Z dimension a small amount.
        let thickness = 0.001;

        let w2 = self.width / 2.0;
        let h2 = self.height / 2.0;

        let offset = match self.axis {
            Axis::X => Vec3::new(thickness, w2, h2),
            Axis::Y => Vec3::new(w2, thickness, h2),
            Axis::Z => Vec3::new(w2, h2, thickness),
        };
        Aabb::new(self.center - offset, self.center + offset)
    }
}

impl Hittable for AxisAlignedRect {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<HitRecord> {
        let bounds = Self::bounding_box(self);
        let span = bounds.span();
        let d0 = self.axis;
        let d1 = d0.next();
        let d2 = d1.next();

        let t = (self.center[d0] - ray.origin[d0]) / ray.direction[d0];

        if t_range.contains(&t) {
            let i = ray.origin[d1] + t * ray.direction[d1];
            let j = ray.origin[d2] + t * ray.direction[d2];
            if !bounds.range(d1).contains(&i) || !bounds.range(d2).contains(&j) {
                None
            } else {
                let u = (i - bounds.min[d1]) / span[d1];
                let v = (j - bounds.min[d2]) / span[d2];
                let outward_normal = d0.into();
                Some(HitRecord::new(
                    ray.along(t),
                    t,
                    ray,
                    outward_normal,
                    self.material.clone(),
                    u,
                    v,
                ))
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _time_range: Range<f64>) -> Option<Aabb> {
        Some(Self::bounding_box(self))
    }
}
