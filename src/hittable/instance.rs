use super::{HitRecord, Hittable};
use crate::geom::{Aabb, Point3, PointCloud, Ray, Vec3};
use std::ops::Range;

#[derive(Clone)]
pub struct Translate {
    pub hittable: Box<dyn Hittable>,
    pub offset: Vec3,
}

impl Hittable for Translate {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<super::HitRecord> {
        let moved = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        self.hittable.hit(moved, t_range).map(|mut hit_record| {
            hit_record.p += self.offset;
            hit_record.set_face_normal(moved, hit_record.normal);
            hit_record
        })
    }

    fn bounding_box(&self, time_range: Range<f64>) -> Option<crate::geom::Aabb> {
        self.hittable
            .bounding_box(time_range)
            .map(|bb| Aabb::new(bb.min + self.offset, bb.max + self.offset))
    }
}

#[derive(Clone)]
pub struct RotateY {
    hittable: Box<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    zero_bbox: Option<Aabb>,
}

impl RotateY {
    pub fn new(hittable: Box<dyn Hittable>, theta: f64) -> Self {
        let (sin_theta, cos_theta) = theta.sin_cos();
        let mut rv = Self {
            hittable,
            sin_theta,
            cos_theta,
            zero_bbox: None,
        };
        rv.zero_bbox = rv.bounding_box(0.0..0.0);
        rv
    }
}

impl Hittable for RotateY {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<super::HitRecord> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;

        origin[0] = self.cos_theta * ray.origin.x() - self.sin_theta * ray.origin.z();
        origin[2] = self.sin_theta * ray.origin.x() + self.cos_theta * ray.origin.z();

        direction[0] = self.cos_theta * ray.direction.x() - self.sin_theta * ray.direction.z();
        direction[2] = self.sin_theta * ray.direction.x() + self.cos_theta * ray.direction.z();

        let rotated = Ray::new(origin, direction, ray.time);

        self.hittable.hit(rotated, t_range).map(|hit_record| {
            let mut p = hit_record.p;
            let mut normal = hit_record.normal;

            p[0] = self.cos_theta * hit_record.p.x() + self.sin_theta * hit_record.p.x();
            p[2] = -self.sin_theta * hit_record.p.x() + self.cos_theta * hit_record.p.x();

            normal[0] =
                self.cos_theta * hit_record.normal.x() + self.sin_theta * hit_record.normal.x();
            normal[2] =
                -self.sin_theta * hit_record.normal.x() + self.cos_theta * hit_record.normal.x();

            let mut rec = HitRecord {
                p,
                normal,
                ..hit_record
            };
            rec.set_face_normal(rotated, normal);
            rec
        })
    }

    fn bounding_box(&self, time_range: Range<f64>) -> Option<Aabb> {
        match (&self.zero_bbox, &time_range) {
            (Some(bbox), tr) if *tr == (0.0..0.0) => Some(bbox.clone()),
            _ => self.hittable.bounding_box(time_range).and_then(|bbox| {
                bbox.corners()
                    .iter()
                    .map(|c| {
                        Point3::new(
                            self.cos_theta * c.x() + self.sin_theta * c.z(),
                            c.y(),
                            -self.sin_theta * c.x() + self.cos_theta * c.z(),
                        )
                    })
                    .collect::<PointCloud>()
                    .bounding_box()
            }),
        }
    }
}
