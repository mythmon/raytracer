use crate::geom::{Point3, Ray, Vec3};
use anyhow::{ensure, Result};
use rand::{thread_rng, Rng};
use std::ops::Range;

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    lens_radius: f64,
    u: Vec3,
    v: Vec3,
    // w: Vec3,
    pub shutter_time: Range<f64>,
}

impl Camera {
    // pub fn new(
    //     look_from: Point3,
    //     look_at: Point3,
    //     v_up: Vec3,
    //     vertical_fov: f64,
    //     aspect_ratio: f64,
    //     aperture: f64,
    //     focus_dist: f64,
    //     shutter_time: Range<f64>,
    // ) -> Self {
    //     let theta = vertical_fov.to_radians();
    //     let h = (theta / 2.0).tan();
    //     let viewport_height = 2.0 * h;
    //     let viewport_width = aspect_ratio * viewport_height;

    //     let w = (look_from - look_at).unit_vector();
    //     let u = v_up.cross(w).unit_vector();
    //     let v = w.cross(u);

    //     let origin = look_from;
    //     let horizontal = focus_dist * viewport_width * u;
    //     let vertical = focus_dist * viewport_height * v;

    //     Self {
    //         origin,
    //         horizontal,
    //         vertical,
    //         lower_left_corner: origin - horizontal / 2.0 - vertical / 2.0 - focus_dist * w,
    //         lens_radius: aperture / 2.0,
    //         u,
    //         v,
    //         shutter_time,
    //     }
    // }

    pub fn build() -> Builder {
        Builder::default()
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let mut rng = thread_rng();
        let rd = self.lens_radius * Vec3::rand_in_unit_disk();
        let offset = self.u * rd.x() + self.v * rd.y();
        let time = if self.shutter_time.end > self.shutter_time.start {
            rng.gen_range(self.shutter_time.clone())
        } else {
            self.shutter_time.start
        };
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
            time,
        )
    }
}

#[derive(Clone, Default)]
pub struct Builder {
    look_from: Option<Point3>,
    look_at: Option<Point3>,
    v_up: Option<Vec3>,
    vertical_fov: Option<f64>,
    aspect_ratio: Option<f64>,
    aperture: Option<f64>,
    focus_dist: Option<f64>,
    shutter_time: Option<Range<f64>>,
}

impl Builder {
    pub fn done(self) -> Result<Camera> {
        ensure!(self.look_at.is_some() || self.look_from.is_some(), "at least one of look_at or look_from are required");
        let look_from = self.look_from.unwrap_or_default();
        let look_at = self.look_at.unwrap_or_default();
        let look_vector = look_from - look_at;

        ensure!(!look_vector.is_near_zero(), "look_at and look_from are too close together");
        let focus_dist = self.focus_dist.unwrap_or_else(|| (look_at - look_from).length());

        let aspect_ratio = self.aspect_ratio.unwrap_or(1.0);
        let theta = self.vertical_fov.unwrap_or(20.0).to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let v_up = self.v_up.unwrap_or(Vec3::new(0.0, 1.0, 0.0));
        let w = look_vector.unit_vector();
        let u = v_up.cross(w).unit_vector();
        let v = w.cross(u);

        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = look_from - horizontal / 2.0 - vertical / 2.0 - focus_dist * w;

        Ok(Camera {
            origin: look_from,
            lower_left_corner,
            horizontal,
            vertical,
            lens_radius: self.aperture.unwrap_or_default() / 2.0,
            u,
            v,
            shutter_time: self.shutter_time.unwrap_or(0.0..0.0),
        })
    }

    pub fn look_from(mut self, look_from: Point3) -> Self {
        self.look_from = Some(look_from);
        self
    }

    pub fn look_at(mut self, look_at: Point3) -> Self {
        self.look_at = Some(look_at);
        self
    }

    pub fn v_up(mut self, v_up: Vec3) -> Self {
        self.v_up = Some(v_up);
        self
    }

    pub fn vertical_fov(mut self, vertical_fov: f64) -> Self {
        self.vertical_fov = Some(vertical_fov);
        self
    }

    pub fn aspect_ratio(mut self, aspect_ratio: f64) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    pub fn aperture(mut self, aperture: f64) -> Self {
        self.aperture = Some(aperture);
        self
    }

    pub fn focus_dist(mut self, focus_dist: f64) -> Self {
        self.focus_dist = Some(focus_dist);
        self
    }

    pub fn shutter_time(mut self, shutter_time: Range<f64>) -> Self {
        self.shutter_time = Some(shutter_time);
        self
    }
}
