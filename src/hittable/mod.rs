mod bvh;
mod hittable_list;
mod moving_sphere;
mod rect;
mod sphere;
mod cuboid;
mod instance;
mod constant_medium;

pub use bvh::BvhNode;
pub use hittable_list::HittableList;
pub use moving_sphere::MovingSphere;
pub use rect::AxisAlignedRect;
pub use sphere::Sphere;
pub use cuboid::Cuboid;
pub use instance::{Translate, RotateY};
pub use constant_medium::ConstantMedium;

use crate::{
    geom::{Aabb, Point3, Ray, Vec3},
    material::Material,
};
use dyn_clonable::clonable;
use std::{ops::Range, sync::Arc};

#[clonable]
pub trait Hittable: Send + Sync + Clone {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<HitRecord>;
    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb>;
}

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub front_face: bool,
    pub u: f64,
    pub v: f64,
}

impl HitRecord {
    pub fn new(
        p: Point3,
        t: f64,
        ray: Ray,
        outward_normal: Vec3,
        material: Arc<dyn Material>,
        u: f64,
        v: f64,
    ) -> Self {
        let mut record = Self {
            p,
            normal: Vec3::default(),
            material,
            t,
            front_face: false,
            u,
            v,
        };
        record.set_face_normal(ray, outward_normal);
        record
    }

    pub fn set_face_normal(&mut self, ray: Ray, outward_normal: Vec3) {
        self.front_face = ray.direction.dot(outward_normal) < 0.0;
        self.normal = if self.front_face { outward_normal } else {-outward_normal}
    }
}
