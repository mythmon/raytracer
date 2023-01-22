use super::{Point3, Vec3};

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub time: f64,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3, time: f64) -> Self {
        Self { origin, direction, time }
    }

    pub fn along(self, portion: f64) -> Point3 {
        self.origin + portion * self.direction
    }
}
