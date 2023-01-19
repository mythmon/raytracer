mod camera;
mod geom;
mod hittable;
mod interpolate;
mod material;
mod scenes;

use crate::{
    camera::Camera,
    geom::{Color, Point3, Ray, Vec3},
};
use anyhow::Result;
use hittable::Hittable;
use indicatif::{ProgressBar, ProgressStyle};
use interpolate::lerp;
use rand::Rng;

fn main() -> Result<()> {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 1200;
    let image_height = (image_width as f64 / aspect_ratio) as u64;
    let samples_per_pixel = 500;
    let max_depth = 50;

    // World
    let world = scenes::random_balls();

    // Camera
    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let camera = Camera::new(
        look_from,
        look_at,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
        aspect_ratio,
        0.1,
        10.0
    );

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
