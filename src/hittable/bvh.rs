use ordered_float::OrderedFloat;

use crate::{
    geom::{Aabb, Axis, Ray},
    hittable::{HitRecord, Hittable, HittableList},
};
use std::ops::Range;

const MAX_PER_LEAF: usize = 256;

/// Bounding Volume Hierarchies
#[derive(Clone)]
pub struct BvhNode {
    time_range: Range<f64>,
    bounding_box: Option<Aabb>,
    contents: BvhContents,
}

#[derive(Clone)]
enum BvhContents {
    Leaf(HittableList),
    Interior {
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
}

impl BvhNode {
    pub fn new(time_range: Range<f64>, hittables: Vec<Box<dyn Hittable>>) -> Self {
        Self::new_along_axis(time_range, hittables, Axis::X)
    }

    fn new_along_axis(
        time_range: Range<f64>,
        mut hittables: Vec<Box<dyn Hittable>>,
        split_axis: Axis,
    ) -> Self {
        if hittables.len() < MAX_PER_LEAF {
            let list = HittableList::new(hittables);
            let bounding_box = list.bounding_box(time_range.clone());
            Self {
                contents: BvhContents::Leaf(list),
                bounding_box,
                time_range,
            }
        } else {
            hittables.sort_by_cached_key(|h| {
                OrderedFloat(if let Some(bbox) = h.bounding_box(time_range.clone()) {
                    (bbox.min[split_axis] + bbox.max[split_axis]) / 2.0
                } else {
                    0.0
                })
            });

            let right = BvhNode::new_along_axis(
                time_range.clone(),
                hittables.split_off(hittables.len() / 2),
                split_axis.next(),
            );
            let left = BvhNode::new_along_axis(time_range.clone(), hittables, split_axis.next());
            let bounding_box =
                if let (Some(lbb), Some(rbb)) = (&left.bounding_box, &right.bounding_box) {
                    Aabb::surrounding(&[lbb, rbb])
                } else {
                    None
                };

            Self {
                contents: BvhContents::Interior {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                bounding_box,
                time_range,
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<HitRecord> {
        if let Some(bounding_box) = &self.bounding_box {
            if !bounding_box.intersect(ray, t_range.clone()) {
                return None;
            }
        }

        match &self.contents {
            BvhContents::Leaf(objects) => objects.hit(ray, t_range),
            BvhContents::Interior { left, right } => {
                match (left.hit(ray, t_range.clone()), right.hit(ray, t_range)) {
                    (Some(hit_left), Some(hit_right)) => {
                        if hit_left.t < hit_right.t {
                            Some(hit_left)
                        } else {
                            Some(hit_right)
                        }
                    }
                    (None, hit @ Some(_)) | (hit @ Some(_), None) => hit,
                    (None, None) => None,
                }
            }
        }
    }

    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb> {
        if self.time_range.start >= time_range.start && self.time_range.end <= time_range.end {
            self.bounding_box.clone()
        } else {
            None
        }
    }
}
