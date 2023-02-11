use ordered_float::OrderedFloat;

use super::{Point3, Aabb, Axis};

pub struct PointCloud(Vec<Point3>);

impl PointCloud {
    pub fn bounding_box(&self) -> Option<Aabb> {
        let mut min = Point3::infinity();
        let mut max = -Point3::infinity();
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            min[axis] = self.0.iter().map(|p| OrderedFloat(p[axis])).min()?.0;
            max[axis] = self.0.iter().map(|p| OrderedFloat(p[axis])).max()?.0;
        }
        Some(Aabb { min, max })
    }
}

impl FromIterator<Point3> for PointCloud {
    fn from_iter<T: IntoIterator<Item = Point3>>(iter: T) -> Self {
        PointCloud(Vec::from_iter(iter))
    }
}
