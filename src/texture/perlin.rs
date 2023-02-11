use crate::{
    geom::{Color, Point3, Vec3},
    texture::Texture,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(Clone)]
pub struct Perlin {
    rand_vec: Vec<Vec3>,
    perm_x: Vec<usize>,
    perm_y: Vec<usize>,
    perm_z: Vec<usize>,
    scale: f64,
}

impl Perlin {
    fn new(scale: f64) -> Self {
        let point_count = 256;
        let mut rng = thread_rng();

        Self {
            rand_vec: std::iter::repeat_with(|| rng.gen::<Vec3>())
                .take(point_count)
                .collect(),
            perm_x: Self::generate_perm(&mut rng, point_count),
            perm_y: Self::generate_perm(&mut rng, point_count),
            perm_z: Self::generate_perm(&mut rng, point_count),
            scale,
        }
    }

    fn generate_perm<R: Rng>(rng: &mut R, point_count: usize) -> Vec<usize> {
        let mut perm = (0..point_count).collect::<Vec<_>>();
        perm.shuffle(rng);
        perm
    }

    fn noise(&self, p: Point3) -> f64 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();
        // hermitian smoothing
        let u = u * u * (3.0 - 2.0 * u);
        let v = v * v * (3.0 - 2.0 * v);
        let w = w * w * (3.0 - 2.0 * w);

        let i = p.x().floor() as isize;
        let j = p.y().floor() as isize;
        let k = p.z().floor() as isize;

        let mut corners = [[[Vec3::default(); 2]; 2]; 2];
        for di in 0..2_isize {
            for dj in 0..2_isize {
                for dk in 0..2_isize {
                    corners[di as usize][dj as usize][dk as usize] = self.rand_vec[self.perm_x
                        [((i + di) & 0xFF) as usize]
                        ^ self.perm_y[((j + dj) & 0xFF) as usize]
                        ^ self.perm_z[((k + dk) & 0xFF) as usize]];
                }
            }
        }
        Self::interp(corners, u, v, w)
    }

    fn turb(&self, mut p: Point3, depth: usize) -> f64 {
        let mut accum = 0.0;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(p);
            weight *= 0.5;
            p *= 2.0;
        }

        accum.abs()
    }

    #[allow(clippy::needless_range_loop)]
    fn interp(corners: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        for i in 0..2_usize {
            for j in 0..2_usize {
                for k in 0..2_usize {
                    let fi = i as f64;
                    let fj = j as f64;
                    let fk = k as f64;
                    let weight_v = Vec3::new(u - fi, v - fj, w - fk);
                    accum += corners[i][j][k].dot(weight_v)
                        * (fi * uu + (1.0 - fi) * (1.0 - uu))
                        * (fj * vv + (1.0 - fj) * (1.0 - vv))
                        * (fk * ww + (1.0 - fk) * (1.0 - ww));
                }
            }
        }
        accum
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new(4.0)
    }
}

impl Texture for Perlin {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        // Color::white() * 0.5 * (1.0 + self.noise(self.scale * p)) // straight noise
        // Color::white() * self.turb(self.scale * p, 7) // turbulent noise
        Color::white() * 0.5 * (1.0 + (self.scale * p.z() + 10.0 * self.turb(p, 7)).sin())
        // marble
    }
}
