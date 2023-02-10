use crate::geom::Axis;
use crate::geom::{Point3, Ray};
use ordered_float::OrderedFloat;
use std::ops::{Range, RangeInclusive};

use super::Vec3;

/// Axis-aligned bounding box
#[derive(Clone, PartialEq, Debug)]
pub struct Aabb {
    pub min: Point3,
    pub max: Point3,
}

impl Aabb {
    pub fn new(a: Point3, b: Point3) -> Self {
        Self {
            min: Point3::new(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2)),
            max: Point3::new(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2)),
        }
    }

    pub fn surrounding(parts: &[&Self]) -> Option<Self> {
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);

        for axis in [Axis::X, Axis::Y, Axis::Z] {
            min[axis] = parts.iter().map(|b| OrderedFloat(b.min[axis])).min()?.0;
            max[axis] = parts.iter().map(|b| OrderedFloat(b.max[axis])).max()?.0;
        }

        Some(Self { min, max })
    }

    pub fn intersect(&self, ray: &Ray, t_range: Range<f64>) -> bool {
        for a in 0..3 {
            let inv_dir = ray.direction[a].recip();
            let mut t0 = (self.min[a] - ray.origin[a]) * inv_dir;
            let mut t1 = (self.max[a] - ray.origin[a]) * inv_dir;
            if inv_dir < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            let t_min = t0.max(t_range.start);
            let t_max = t1.min(t_range.end);
            if t_max <= t_min {
                return false;
            }
        }
        return true;
    }

    pub fn span(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn range(&self, axis: Axis) -> RangeInclusive<f64> {
        match axis {
            Axis::X => self.x_range(),
            Axis::Y => self.y_range(),
            Axis::Z => self.z_range(),
        }
    }

    pub fn x_range(&self) -> RangeInclusive<f64> {
        self.min.x()..=self.max.x()
    }

    pub fn y_range(&self) -> RangeInclusive<f64> {
        self.min.y()..=self.max.y()
    }

    pub fn z_range(&self) -> RangeInclusive<f64> {
        self.min.z()..=self.max.z()
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::{Aabb, Point3};

    #[test]
    fn test_surrounding() {
        let a = Aabb::new(Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, 5.0, 6.0));
        let b = Aabb::new(Point3::new(7.0, 8.0, 9.0), Point3::new(10.0, 11.0, 12.0));
        let c = Aabb::surrounding(&[&a, &b]);
        assert_eq!(
            c,
            Some(Aabb::new(
                Point3::new(1.0, 2.0, 3.0),
                Point3::new(10.0, 11.0, 12.0)
            ))
        );
    }
}
