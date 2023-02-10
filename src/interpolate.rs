use std::ops::{Add, Mul};

pub fn lerp<T>(a: T, b: T, t: f64) -> T
where
    f64: Mul<T, Output = T>,
    T: Add<T, Output = T>,
{
    (1.0 - t) * a + t * b
}

#[allow(dead_code, clippy::needless_range_loop)]
pub fn trilinear_interp(corners: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
    let mut acc = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                let fi = i as f64;
                let fj = j as f64;
                let fk = k as f64;
                acc += (fi * u + (1.0 - fi) * (1.0 - u))
                    * (fj * v + (1.0 - fj) * (1.0 - v))
                    * (fk * w + (1.0 - fk) * (1.0 - w))
                    * corners[i][j][k];
            }
        }
    }
    acc
}
