use crate::{
    geom::{Color, Point3, Vec3},
    hittable::{self, HittableList},
    material, camera::Camera, config::{self, Scene},
};
use anyhow::{Context, Result};
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Scene")]
struct SceneDesc {
    materials: HashMap<String, MaterialDesc>,
    objects: Vec<HittableDesc>,
    camera: CameraDesc,
    image: config::ImageConfig
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Material")]
enum MaterialDesc {
    Lambertian { albedo: Color },
    Dielectric { index_of_refraction: f64 },
    Metal { albedo: Color, fuzziness: f64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Object")]
enum HittableDesc {
    Sphere {
        material: String,
        center: Point3,
        radius: f64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Camera")]
struct CameraDesc {
    look_from: Point3,
    look_at: Option<Point3>,
    v_up: Option<Vec3>,
    vertical_fov: f64,
    aperture: f64,
    focus_distance: f64,
}

pub fn load_scene(path: &Path) -> Result<Scene> {
    let f = std::fs::File::open(path).context("opening scene file")?;

    let ron_options = ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME);
    let scene_desc: SceneDesc = ron_options.from_reader(f).context("loading scene file")?;

    let mut materials: HashMap<String, Arc<dyn material::Material>> = HashMap::new();
    for (key, desc) in scene_desc.materials.into_iter() {
        materials.insert(
            key,
            match desc {
                MaterialDesc::Lambertian { albedo } => Arc::new(material::Lambertian { albedo }),
                MaterialDesc::Dielectric {
                    index_of_refraction,
                } => Arc::new(material::Dielectric {
                    index_of_refraction,
                }),
                MaterialDesc::Metal { albedo, fuzziness } => {
                    Arc::new(material::Metal { albedo, fuzziness })
                }
            },
        );
    }

    let mut world = HittableList::default();
    for desc in scene_desc.objects.into_iter() {
        world.add(match desc {
            HittableDesc::Sphere {
                material,
                center,
                radius,
            } => hittable::Sphere {
                center,
                radius,
                material: materials
                    .get(&material)
                    .ok_or_else(|| anyhow::anyhow!("Material {} not defined", material))?
                    .clone(),
            },
        });
    }

    let aspect_ratio = scene_desc.image.width as f64 / scene_desc.image.height as f64;

    let camera = Camera::new(
        scene_desc.camera.look_from,
        scene_desc.camera.look_at.unwrap_or_default(),
        scene_desc.camera.v_up.unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0)),
        scene_desc.camera.vertical_fov,
        aspect_ratio,
        scene_desc.camera.aperture,
        scene_desc.camera.focus_distance
    );

    Ok(Scene {world, camera, image: scene_desc.image})
}
