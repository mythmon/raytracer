use std::sync::{Arc};

use crate::{
    geom::{Color, Point3},
    hittable::{self, HittableList},
    material,
};

#[allow(dead_code)]
pub fn scene() -> HittableList {
    let mut world = hittable::HittableList::default();

    let material_ground = Arc::new(material::Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    });
    let material_center = Arc::new(material::Lambertian {
        albedo: Color::new(0.1, 0.2, 0.5),
    });
    let material_left = Arc::new(material::Dielectric {
        index_of_refraction: 1.5,
    });
    let material_right = Arc::new(material::Metal {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzziness: 0.0,
    });

    world.add(hittable::Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: material_ground,
    });
    world.add(hittable::Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: material_center,
    });
    world.add(hittable::Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: 0.5,
        material: material_left.clone(),
    });
    world.add(hittable::Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: -0.4,
        material: material_left,
    });
    world.add(hittable::Sphere {
        center: Point3::new(1.0, 0.0, -1.0),
        radius: 0.5,
        material: material_right,
    });
    world
}
