use crate::geom::Axis;
use image::Rgb;
use pix::rgb::SRgb8;
use rand::{distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};
use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Copy, Clone, Default, Deserialize, Serialize, Debug, PartialEq)]
pub struct Vec3(pub f64, pub f64, pub f64);

pub type Color = Vec3;
pub type Point3 = Vec3;

impl Vec3 {
    pub fn new(e1: f64, e2: f64, e3: f64) -> Self {
        Self(e1, e2, e3)
    }

    pub fn white() -> Self {
        Self(1.0, 1.0, 1.0)
    }

    pub fn black() -> Self {
        Self(0.0, 0.0, 0.0)
    }

    pub fn infinity() -> Self {
        let inf = f64::INFINITY;
        Self(inf, inf, inf)
    }

    pub fn rand_unit_vector() -> Self {
        let between = Uniform::new_inclusive(-1.0, 1.0);
        let mut rng = rand::thread_rng();
        loop {
            let x = between.sample(&mut rng);
            let y = between.sample(&mut rng);
            let z = between.sample(&mut rng);
            let v = Vec3(x, y, z);
            if v.length_squared() < 1.0 {
                break v.unit_vector();
            }
        }
    }

    pub fn rand_in_unit_disk() -> Self {
        let between = Uniform::new_inclusive(-1.0, 1.0);
        let mut rng = rand::thread_rng();
        loop {
            let x = between.sample(&mut rng);
            let y = between.sample(&mut rng);
            let v = Vec3(x, y, 0.0);
            if v.length_squared() < 1.0 {
                break v;
            }
        }
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn dot(self, rhs: Vec3) -> f64 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
    }

    pub fn cross(self, rhs: Vec3) -> Vec3 {
        Self(
            self.1 * rhs.2 - self.2 * rhs.1,
            self.2 * rhs.0 - self.0 * rhs.2,
            self.0 * rhs.1 - self.1 * rhs.0,
        )
    }

    /// Reflect this vector across a normal vector
    pub fn reflect(self, normal: Vec3) -> Vec3 {
        self - 2.0 * self.dot(normal) * normal
    }

    pub fn refract(self, normal: Vec3, eta_i_over_eta_t: f64) -> Vec3 {
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perp = eta_i_over_eta_t * (self + cos_theta * normal);
        let r_out_parallel = (1.0 - r_out_perp.length_squared()).abs().sqrt() * -normal;
        r_out_perp + r_out_parallel
    }

    pub fn unit_vector(self) -> Vec3 {
        self / self.length()
    }

    pub fn x(self) -> f64 {
        self.0
    }
    pub fn y(self) -> f64 {
        self.1
    }
    pub fn z(self) -> f64 {
        self.2
    }
    pub fn r(self) -> f64 {
        self.0
    }
    pub fn g(self) -> f64 {
        self.1
    }
    pub fn b(self) -> f64 {
        self.2
    }

    /// Checks if a vector is close to zero in all dimensions
    pub fn is_near_zero(&self) -> bool {
        const LIMIT: f64 = 0.001;
        self.0.abs() < LIMIT && self.1.abs() < LIMIT && self.2.abs() < LIMIT
    }

    pub fn into_srgb8(self, samples_per_pixel: u32) -> SRgb8 {
        let scale = (samples_per_pixel as f64).recip();
        let max = 255.0 / 256.0;
        let r = (self.r() * scale).clamp(0.0, max);
        let g = (self.g() * scale).clamp(0.0, max);
        let b = (self.b() * scale).clamp(0.0, max);
        let ir = (r * 256.0) as u8;
        let ig = (g * 256.0) as u8;
        let ib = (b * 256.0) as u8;
        SRgb8::new(ir, ig, ib)
    }

    pub fn project(self, rhs: Self) -> Self {
        let rhs_unit = rhs.unit_vector();
        rhs_unit * self.dot(rhs_unit)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Self::Output {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Self::Output {
        self + -rhs
    }
}

impl SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self += -rhs;
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl MulAssign<Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: Vec3) {
        self.0 *= rhs.0;
        self.1 *= rhs.1;
        self.2 *= rhs.2;
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        self * rhs.recip()
    }
}

impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, rhs: f64) {
        self.mul_assign(rhs.recip());
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| a + b).unwrap_or_default()
    }
}

impl Distribution<Vec3> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        Vec3(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        )
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("invalid axis {index} for Vec3"),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("invalid axis {index} for Vec3"),
        }
    }
}

impl Index<Axis> for Vec3 {
    type Output = f64;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => &self.0,
            Axis::Y => &self.1,
            Axis::Z => &self.2,
        }
    }
}

impl IndexMut<Axis> for Vec3 {
    fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
        match index {
            Axis::X => &mut self.0,
            Axis::Y => &mut self.1,
            Axis::Z => &mut self.2,
        }
    }
}

impl From<&Rgb<u8>> for Color {
    fn from(val: &Rgb<u8>) -> Self {
        let scale = 1.0 / 255.0;
        let r = val.0[0] as f64 * scale;
        let g = val.0[1] as f64 * scale;
        let b = val.0[2] as f64 * scale;
        Vec3(r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use crate::geom::Vec3;

    #[test]
    fn test_project() {
        assert_eq!(Vec3(1., 2., 3.).project(Vec3(2., 0., 0.)), Vec3(1., 0., 0.));
        assert_eq!(Vec3(1., 2., 3.).project(Vec3(0., 2., 0.)), Vec3(0., 2., 0.));
        assert_eq!(Vec3(1., 2., 3.).project(Vec3(0., 0., 2.)), Vec3(0., 0., 3.));
    }
}