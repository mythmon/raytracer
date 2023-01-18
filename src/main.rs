pub mod camera;
pub mod geom;
pub mod hittable;
pub mod interpolate;
pub mod material;

use std::rc::Rc;

use crate::{
    camera::Camera,
    geom::{Color, Point3, Ray},
    hittable::{hittable_list::HittableList, sphere::Sphere},
    material::{Lambertian, Metal},
};
use anyhow::Result;
use hittable::Hittable;
use indicatif::{ProgressBar, ProgressStyle};
use interpolate::lerp;
use rand::Rng;

fn main() -> Result<()> {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 800;
    let image_height = (image_width as f64 / aspect_ratio) as u64;
    let samples_per_pixel = 100;
    let max_depth = 50;

    // World
    let mut world = HittableList::default();

    let material_ground = Rc::new(Lambertian {
        albedo: Color::new(0.8, 0.8, 0.0),
    });
    let material_center = Rc::new(Lambertian {
        albedo: Color::new(0.7, 0.3, 0.3),
    });
    let material_left = Rc::new(Metal {
        albedo: Color::new(0.8, 0.8, 0.8),
        fuzziness: 0.1,
    });
    let material_right = Rc::new(Metal {
        albedo: Color::new(0.8, 0.6, 0.2),
        fuzziness: 0.5,
    });

    world.add(Rc::new(Sphere {
        center: Point3::new(0.0, -100.5, -1.0),
        radius: 100.0,
        material: material_ground,
    }));
    world.add(Rc::new(Sphere {
        center: Point3::new(0.0, 0.0, -1.0),
        radius: 0.5,
        material: material_center,
    }));
    world.add(Rc::new(Sphere {
        center: Point3::new(-1.0, 0.0, -1.0),
        radius: 0.5,
        material: material_left,
    }));
    world.add(Rc::new(Sphere {
        center: Point3::new(1.0, 0.0, -1.0),
        radius: 0.5,
        material: material_right,
    }));

    // Camera
    let camera = Camera::default();

    // Render
    println!("P3"); // pixel format
    println!("{} {}", image_width, image_height); // image size
    println!("255"); // max color value

    let bar = ProgressBar::new(image_height);
    bar.set_style(ProgressStyle::with_template(
        "{bar} [{elapsed}/{duration}] {msg}",
    )?);

    let mut rng = rand::thread_rng();
    for j in (0..image_height).rev() {
        for i in 0..image_width {
            let color: Color = (0..samples_per_pixel)
                .into_iter()
                .map(|_| {
                    let u = (i as f64 + rng.gen::<f64>()) / (image_width as f64 - 1.0);
                    let v = (j as f64 + rng.gen::<f64>()) / (image_height as f64 - 1.0);
                    let ray = camera.get_ray(u, v);
                    ray_color(ray, &world, max_depth)
                })
                .sum();
            println!("{}", color.into_ppm_color(samples_per_pixel));
        }
        bar.inc(1);
        bar.set_message(format!("{} scanlines remaining", j));
    }
    bar.finish();

    Ok(())
}

fn ray_color<H: Hittable>(ray: Ray, hittable: &H, depth_budget: usize) -> Color {
    if depth_budget == 0 {
        Color::default()
    } else if let Some(hit_record) = hittable.hit(&ray, 0.001..f64::INFINITY) {
        let scatter_record = hit_record.material.scatter(&ray, &hit_record);
        if let Some(scattered) = scatter_record.scattered_ray {
            return scatter_record.attenuation * ray_color(scattered, hittable, depth_budget - 1);
        } else {
            Color::default()
        }
    } else {
        let unit = ray.direction.unit_vector();
        let t = 0.5 * (unit.y() + 1.0);
        lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t) // blue
                                                                      // lerp(Color::new(1.0, 1.0, 1.0), Color::new(1.0, 0.3, 0.1), t) // red
    }
}
