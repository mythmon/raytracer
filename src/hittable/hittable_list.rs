use std::ops::Range;

use crate::{geom::Aabb, hittable::Hittable};
use ordered_float::OrderedFloat;

#[derive(Clone, Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new(objects: Vec<Box<dyn Hittable>>) -> Self {
        Self { objects }
    }

    #[allow(dead_code)]
    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &crate::geom::Ray, t_range: Range<f64>) -> Option<super::HitRecord> {
        // the book does this by maintaining a single hittable record which is
        // passed by reference into `hit()`, which can update it and return if a
        // hit was found. That's arguably more efficient. This is way simpler.
        // Consider optimizing this if things are too slow though.
        self.objects
            .iter()
            .filter_map(|h| h.hit(ray, t_range.clone()))
            .min_by_key(|hr| OrderedFloat(hr.t))
    }

    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb> {
        // If any of the objects has an undefined bounding box, propogate that.
        let bounding_boxes: Option<Vec<_>> = self
            .objects
            .iter()
            .map(|o| o.bounding_box(time_range.clone()))
            .collect();
        bounding_boxes.and_then(|bbs| Aabb::surrounding(&(bbs.iter().collect::<Vec<_>>())))
    }
}
