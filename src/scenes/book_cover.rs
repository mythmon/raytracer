use crate::{
    geom::{Color, Point3},
    hittable::{self, HittableList, Sphere},
    material::{self, Material},
};
use rand::Rng;
use std::rc::Rc;

#[allow(dead_code)]
pub fn scene() -> HittableList {
    let mut world = HittableList::default();
    let mut rng = rand::thread_rng();

    let ground_material: Rc<dyn Material> = Rc::new(material::Lambertian {
        albedo: Color::new(0.5, 0.5, 0.5),
    });
    world.add(Rc::new(Sphere {
        center: Point3::new(0.0, -1000.0, 0.0),
        radius: 1000.0,
        material: ground_material,
    }));

    let glass_material = Rc::new(material::Dielectric {
        index_of_refraction: 1.5,
    });

    let point_of_interest = Point3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new(
                a as f64 + rng.gen_range(0.0..0.9),
                0.2,
                b as f64 + rng.gen_range(0.0..0.9),
            );

            if (center - point_of_interest).length() > 0.9 {
                let material: Rc<dyn Material> = if choose_mat < 0.8 {
                    Rc::new(material::Lambertian { albedo: rng.gen() })
                } else if choose_mat < 0.95 {
                    Rc::new(material::Metal {
                        albedo: rng.gen(),
                        fuzziness: rng.gen_range(0.0..0.5),
                    })
                } else {
                    glass_material.clone()
                };
                world.add(Rc::new(Sphere {
                    center,
                    radius: 0.2,
                    material,
                }));
            }
        }
    }

    world.add(Rc::new(hittable::Sphere {
        center: Point3::new(0.0, 1.0, 0.0),
        radius: 1.0,
        material: glass_material,
    }));

    let diffuse_material = Rc::new(material::Lambertian {
        albedo: Color::new(0.4, 0.2, 0.1),
    });
    world.add(Rc::new(hittable::Sphere {
        center: Point3::new(-4.0, 1.0, 0.0),
        radius: 1.0,
        material: diffuse_material,
    }));

    let metal_material = Rc::new(material::Metal {
        albedo: Color::new(0.7, 0.6, 0.5),
        fuzziness: 0.0,
    });
    world.add(Rc::new(hittable::Sphere {
        center: Point3::new(4.0, 1.0, 0.0),
        radius: 1.0,
        material: metal_material,
    }));

    world
}
