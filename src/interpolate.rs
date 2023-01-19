use std::ops::{Add, Mul};

pub fn lerp<T>(a: T, b: T, t: f64) -> T
where
    f64: Mul<T, Output = T>,
    T: Add<T, Output = T>,
{
    (1.0 - t) * a + t * b
}
