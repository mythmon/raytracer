mod hittable_list;
mod sphere;
mod moving_sphere;
mod bvh;

pub use hittable_list::HittableList;
pub use sphere::Sphere;
pub use moving_sphere::MovingSphere;
pub use bvh::BvhNode;

use crate::{
    geom::{Point3, Ray, Vec3, Aabb},
    material::Material,
};
use dyn_clonable::clonable;
use std::{ops::Range, sync::Arc};

#[clonable]
pub trait Hittable: Send + Sync + Clone {
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord>;
    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb>;
}

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Point3,
        t: f64,
        ray: &Ray,
        outward_normal: Vec3,
        material: Arc<dyn Material>,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            p,
            normal,
            material,
            t,
            front_face,
        }
    }
}
