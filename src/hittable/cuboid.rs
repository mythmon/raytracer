use crate::{
    geom::{Aabb, Axis, Point3, Vec3, Ray},
    hittable::{AxisAlignedRect, Hittable},
    material::Material,
};
use std::{ops::Range, sync::Arc};

use super::HittableList;

#[derive(Clone)]
pub struct Cuboid {
    center: Point3,
    half_size: Vec3,
    sides: HittableList,
}

impl Cuboid {
    pub fn new(center: Point3, size: Vec3, material: &Arc<dyn Material>) -> Self {
        let half_size = size / 2.0;

        let sides: HittableList = itertools::iproduct!([Axis::X, Axis::Y, Axis::Z], [-1.0, 1.0])
            .into_iter()
            .map(|(axis, mult)| AxisAlignedRect {
                axis,
                center: center + half_size.project(axis.into()) * mult,
                width: size[axis.next()],
                height: size[axis.prev()],
                material: material.clone(),
            })
            .collect();

        Self {
            center,
            half_size,
            sides,
        }
    }
}

impl Hittable for Cuboid {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<super::HitRecord> {
        self.sides.hit(ray, t_range)
    }

    fn bounding_box(&self, _time_range: Range<f64>) -> Option<Aabb> {
        Some(Aabb::new(self.center - self.half_size, self.center + self.half_size))
    }
}
