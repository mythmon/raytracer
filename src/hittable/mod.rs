pub mod hittable_list;
pub mod sphere;

use crate::{geom::{Point3, Vec3, Ray}, material::Material};
use std::{ops::Range, rc::Rc};

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord>;
}

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Rc<dyn Material>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(p: Point3, t: f64, ray: &Ray, outward_normal: Vec3, material: Rc<dyn Material>) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self { p, normal, material, t, front_face }
    }
}