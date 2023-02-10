mod camera;
mod config;
mod geom;
mod hittable;
mod interpolate;
mod loader;
mod material;
mod texture;

use crate::{
    geom::{Color, Ray},
    hittable::BvhNode, loader::SceneLoader,
};
use anyhow::{Context, Result};
use clap::Parser;
use hittable::Hittable;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use interpolate::lerp;
use pix::rgb::SRgb8;
use png_pong::PngRaster;
use rand::{distributions, prelude::Distribution};
use rayon::prelude::ParallelIterator;
use std::{
    path::PathBuf,
    time::{self, Duration},
};

#[derive(Parser)]
struct Args {
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Scene
    let loader = SceneLoader::new(&args.path);
    let config::Scene {
        world,
        camera,
        image,
    } = loader.load()?;

    // Render
    let bar = ProgressBar::new(image.height as u64 * image.width as u64);
    bar.set_style(ProgressStyle::with_template(
        "{bar} {human_pos}/{human_len} ({percent}%) {elapsed_precise}",
    )?);

    let start = time::Instant::now();

    let mut raster = pix::Raster::<SRgb8>::with_clear(image.width, image.height);
    struct LocatedPixel<'a> {
        x: usize,
        y: usize,
        pixel: &'a mut SRgb8,
    }
    struct ParallelWorkItem<'a> {
        pixels: &'a mut [LocatedPixel<'a>],
        world: BvhNode,
    }

    let mut pixels = raster
        .pixels_mut()
        .iter_mut()
        .enumerate()
        .map(|(index, pixel)| LocatedPixel {
            x: index % image.width as usize,
            y: index / image.width as usize,
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
                    .take(image.samples_per_pixel as usize)
                    .map(|(jx, jy): (f64, f64)| {
                        let u = (*x as f64 + jx) / (image.width as f64 - 1.0);
                        let v = ((image.height as usize - *y) as f64 + jy)
                            / (image.height as f64 - 1.0);
                        let ray = camera.get_ray(u, v);
                        ray_color(ray, &world, image.max_depth)
                    })
                    .sum();
                **pixel = color.into_srgb8(image.samples_per_pixel);
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

fn ray_color<H: Hittable>(ray: Ray, hittable: &H, depth_budget: u32) -> Color {
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
