use std::rc::Rc;
use ordered_float::OrderedFloat;
use crate::hittable::Hittable;

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Rc<dyn Hittable>>,
}

impl HittableList {
    pub fn add<T: Hittable + 'static>(&mut self, object: Rc<T>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(
        &self,
        ray: &crate::geom::Ray,
        t_range: std::ops::Range<f64>,
    ) -> Option<super::HitRecord> {
        // the book does this by maintaining a single hittable record which is
        // passed by reference into `hit()`, which can update it and return if a
        // hit was found. That's arguably more efficient. This is way simpler.
        // Consider optimizing this if things are too slow though.
        self.objects
            .iter()
            .flat_map(|h| h.hit(ray, t_range.clone()))
            .min_by_key(|hr| OrderedFloat(hr.t))
    }
}
