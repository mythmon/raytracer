mod camera;
mod geom;
mod hittable;
mod interpolate;
mod material;
mod scenes;
use crate::{
    camera::Camera,
    geom::{Color, Point3, Ray, Vec3},
    hittable::HittableList,
};
use anyhow::{Context, Result};
use hittable::Hittable;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use interpolate::lerp;
use pix::rgb::SRgb8;
use png_pong::PngRaster;
use rand::{distributions, prelude::Distribution};
use rayon::prelude::ParallelIterator;
use std::time::{self, Duration};

fn main() -> Result<()> {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 1200;
    let image_height = (image_width as f64 / aspect_ratio) as u32;
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
        10.0,
    );

    // Render
    let bar = ProgressBar::new(image_height as u64 * image_width as u64);
    bar.set_style(ProgressStyle::with_template(
        "{bar} {human_pos}/{human_len} ({percent}%) {elapsed_precise}",
    )?);

    let start = time::Instant::now();

    let mut raster = pix::Raster::<SRgb8>::with_clear(image_width, image_height);
    struct LocatedPixel<'a> {
        x: usize,
        y: usize,
        pixel: &'a mut SRgb8,
    }
    struct ParallelWorkItem<'a> {
        pixels: &'a mut [LocatedPixel<'a>],
        world: HittableList,
    }

    let mut pixels = raster
        .pixels_mut()
        .iter_mut()
        .enumerate()
        .map(|(index, pixel)| LocatedPixel {
            x: index % image_width as usize,
            y: index / image_width as usize,
            pixel,
        })
        .collect::<Vec<_>>();
    let work = ParallelWorkItem {
        pixels: &mut pixels[..],
        world,
    };

    fn split_pixels<'a>(
        work: ParallelWorkItem<'a>,
    ) -> (ParallelWorkItem<'a>, Option<ParallelWorkItem<'a>>) {
        let h = work.pixels.len() / 2;
        if h > 0 {
            let (left, right) = work.pixels.split_at_mut(h);
            (
                ParallelWorkItem {
                    pixels: left,
                    world: work.world.clone(),
                },
                Some(ParallelWorkItem {
                    pixels: right,
                    world: work.world,
                }),
            )
        } else {
            (work, None)
        }
    }

    rayon::iter::split(work, split_pixels)
        .progress_with(bar.clone())
        .for_each(|ParallelWorkItem { world, pixels }| {
            let mut rng = rand::thread_rng();
            for LocatedPixel { x, y, pixel } in pixels.into_iter() {
                let color: Color = distributions::Standard
                    .sample_iter(&mut rng)
                    .take(samples_per_pixel)
                    .map(|(jx, jy): (f64, f64)| {
                        let u = (*x as f64 + jx) / (image_width as f64 - 1.0);
                        let v = ((image_height as usize - *y) as f64 + jy)
                            / (image_height as f64 - 1.0);
                        let ray = camera.get_ray(u, v);
                        ray_color(ray, &world, max_depth)
                    })
                    .sum();
                **pixel = color.into_srgb8(samples_per_pixel);
                bar.inc(1);
            }
        });

    // Saving raster as a PNG file
    let png_raster = PngRaster::Rgb8(raster);
    let mut out_data = Vec::new();
    let mut encoder = png_pong::Encoder::new(&mut out_data).into_step_enc();
    let step = png_pong::Step {
        raster: png_raster,
        delay: 0,
    };
    encoder.encode(&step).context("Adding frame to png")?;
    std::fs::write("image.png", out_data).context("Saving image")?;

    let duration = time::Instant::now().saturating_duration_since(start);
    println!("Done in {}", human_duration(duration));

    Ok(())
}

fn ray_color<H: Hittable>(ray: Ray, hittable: &H, depth_budget: usize) -> Color {
    if depth_budget == 0 {
        Color::default()
    } else if let Some(hit_record) = hittable.hit(&ray, 0.001..f64::INFINITY) {
        let scatter_record = {
            let material = hit_record.material.clone();
            material.scatter(&ray, &hit_record)
        };
        if let Some(scattered) = scatter_record.scattered_ray {
            return scatter_record.attenuation * ray_color(scattered, hittable, depth_budget - 1);
        } else {
            Color::default()
        }
    } else {
        let unit = ray.direction.unit_vector();
        let t = 0.5 * (unit.y() + 1.0);
        lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), t) // blue
    }
}

const SECOND: u128 = 1000;
const MINUTE: u128 = SECOND * 60;
const HOUR: u128 = MINUTE * 60;

fn human_duration(d: Duration) -> String {
    let mut parts = vec![];
    let mut millis = d.as_millis();

    if millis > HOUR {
        parts.push(format!("{}h", millis / HOUR));
        millis %= HOUR;
    }
    if millis > MINUTE || parts.len() > 0 {
        parts.push(format!("{}m", millis / MINUTE));
        millis %= MINUTE;
    }
    parts.push((millis / SECOND).to_string());
    millis %= SECOND;
    if parts.len() == 1 {
        parts.push(format!(".{:0>2}", millis / 10));
    }
    parts.push("s".to_string());
    parts.join("")
}
