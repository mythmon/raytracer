use crate::{
    camera::Camera,
    config::Scene,
    geom::{Color, Vec3},
    hittable::{self, AxisAlignedRect, BvhNode, Hittable},
    material,
    scene::desc,
    texture::{self, Texture},
};
use anyhow::{anyhow, Context, Result};
use rand::{prelude::Distribution, thread_rng, Rng};
use ron::extensions::Extensions;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct SceneLoader {
    pub(crate) scene_path: PathBuf,
    pub(crate) pattern_vars: HashMap<String, i32>,
    pub(crate) materials: HashMap<String, Arc<dyn material::Material>>,
}

impl SceneLoader {
    pub fn new(path: &Path) -> Self {
        Self {
            scene_path: path.into(),
            pattern_vars: HashMap::default(),
            materials: HashMap::default(),
        }
    }

    pub fn load(mut self) -> Result<Scene<BvhNode>> {
        let f = std::fs::File::open(&self.scene_path).context("opening scene file")?;

        let ron_options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let scene_desc: desc::SceneDesc =
            ron_options.from_reader(f).context("loading scene file")?;

        for (key, desc) in scene_desc.materials.into_iter() {
            let material = self.realize_material(desc)?;
            self.materials.insert(key, material);
        }

        let mut hittables: Vec<Box<dyn Hittable>> = vec![];
        for desc in scene_desc.objects.into_iter() {
            match desc {
                desc::HittableDesc::Sphere {
                    material,
                    center,
                    radius,
                } => hittables.push(Box::new(hittable::Sphere {
                    center: self.eval_vec3(center)?,
                    radius: radius.eval(&self)?,
                    material: self.realize_material(material)?,
                })),
                desc::HittableDesc::MovingSphere {
                    center,
                    time,
                    radius,
                    material,
                } => hittables.push(Box::new(hittable::MovingSphere {
                    center: (self.eval_vec3(center.0)?)..(self.eval_vec3(center.1)?),
                    time: (time.0.eval(&self)?)..(time.1.eval(&self)?),
                    radius: radius.eval(&self)?,
                    material: self.realize_material(material)?,
                })),
                desc::HittableDesc::AARect {
                    center,
                    width,
                    height,
                    axis,
                    material,
                } => hittables.push(Box::new(AxisAlignedRect {
                    center: self.eval_vec3(center)?,
                    width: width.eval(&self)?,
                    height: height.eval(&self)?,
                    axis: axis.into(),
                    material: self.realize_material(material)?,
                })),
                desc::HittableDesc::Pattern { var, range, object } => {
                    self.realize_pattern(&mut hittables, var, &range[..], &*object)?;
                }
            };
        }

        let aspect_ratio = scene_desc.image.width as f64 / scene_desc.image.height as f64;
        let look_from = scene_desc.camera.look_from;
        let look_at = scene_desc.camera.look_at.unwrap_or_default();

        let camera = Camera::new(
            look_from,
            look_at,
            scene_desc
                .camera
                .v_up
                .unwrap_or_else(|| Vec3::new(0.0, 1.0, 0.0)),
            scene_desc.camera.vertical_fov,
            aspect_ratio,
            scene_desc.camera.aperture,
            scene_desc
                .camera
                .focus_distance
                .unwrap_or_else(|| (look_at - look_from).length()),
            scene_desc
                .camera
                .shutter_time
                .map(|(a, b)| a..b)
                .unwrap_or(0.0..0.0),
        );

        Ok(Scene {
            world: BvhNode::new(camera.shutter_time.clone(), hittables),
            camera,
            image: scene_desc.image,
            background: scene_desc
                .background
                .map(|v| self.eval_vec3(v))
                .unwrap_or_else(|| Ok(Color::black()))?,
        })
    }

    pub(crate) fn realize_pattern(
        &mut self,
        hittables: &mut Vec<Box<dyn Hittable>>,
        var: String,
        range: &[i32],
        object: &desc::HittableDesc,
    ) -> Result<()> {
        let range = match range {
            &[end] => (0..end).step_by(1),
            &[start, end] => (start..end).step_by(1),
            &[start, end, step] => (start..end).step_by(step as usize),
            unexpected => {
                anyhow::bail!("Unexpected format for range: {:?}", unexpected);
            }
        };

        for val in range {
            self.pattern_vars.insert(var.to_string(), val);
            match *object {
                desc::HittableDesc::Sphere {
                    ref center,
                    ref radius,
                    ref material,
                } => hittables.push(Box::new(hittable::Sphere {
                    center: self.eval_vec3(center.clone())?,
                    radius: radius.eval(&self)?,
                    material: self.realize_material((*material).clone())?,
                })),

                desc::HittableDesc::MovingSphere {
                    ref center,
                    ref time,
                    ref radius,
                    ref material,
                } => {
                    let c1 = self.eval_vec3(center.0.clone())?;
                    let c2 = self.eval_vec3(center.1.clone())?;
                    hittables.push(Box::new(hittable::MovingSphere {
                        center: c1..c2,
                        time: (time.0.eval(&self)?)..(time.1.eval(&self)?),
                        radius: radius.eval(&self)?,
                        material: self.realize_material((*material).clone())?,
                    }))
                }

                desc::HittableDesc::AARect {
                    ref center,
                    ref width,
                    ref height,
                    ref axis,
                    ref material,
                } => hittables.push(Box::new(AxisAlignedRect {
                    center: self.eval_vec3(center.clone())?,
                    width: width.eval(&self)?,
                    height: height.eval(&self)?,
                    axis: (*axis).into(),
                    material: self.realize_material((*material).clone())?,
                })),

                desc::HittableDesc::Pattern {
                    ref var,
                    ref range,
                    ref object,
                } => {
                    self.realize_pattern(hittables, var.clone(), range, object)?;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn realize_material<D: Into<desc::MaterialDesc>>(
        &self,
        desc: D,
    ) -> Result<Arc<dyn material::Material>> {
        let desc: desc::MaterialDesc = desc.into();
        Ok(match desc {
            desc::MaterialDesc::Shared(ref name) => self
                .materials
                .get(name)
                .ok_or_else(|| anyhow!("Material {} not defined", name))?
                .clone(),
            desc::MaterialDesc::Lambertian { albedo } => Arc::new(material::Lambertian {
                albedo: self.realize_texture(albedo)?,
            }),
            desc::MaterialDesc::Dielectric {
                index_of_refraction,
            } => Arc::new(material::Dielectric {
                index_of_refraction: index_of_refraction
                    .eval(&self)
                    .context("evaluating index_of_refraction")?,
            }),
            desc::MaterialDesc::Metal { albedo, fuzziness } => Arc::new(material::Metal {
                albedo: self.eval_vec3(albedo)?,
                fuzziness: fuzziness.eval(&self).context("evaluating fuzziness")?,
            }),
            desc::MaterialDesc::DiffuseLight { color } => Arc::new(material::DiffuseLight {
                texture: self.realize_texture(color)?,
            }),
            desc::MaterialDesc::RandomChoice(options) => {
                let mut rng = thread_rng();
                let idx = rng.gen_range(0..options.len());
                self.realize_material((*options[idx]).clone())?
            }
            desc::MaterialDesc::RandomChoiceWeighted(options) => {
                let dist = rand::distributions::WeightedIndex::new(options.iter().map(|c| c.0))
                    .context("generating weighted distribution")?;
                let mut rng = thread_rng();
                let idx = dist.sample(&mut rng);
                self.realize_material((*options[idx].1).clone())?
            }
        })
    }

    pub(crate) fn realize_texture(&self, desc: desc::TextureDesc) -> Result<Box<dyn Texture>> {
        Ok(match desc {
            desc::TextureDesc::Solid(r, g, b) => Box::new(texture::SolidColor(Color::new(r, g, b))),
            desc::TextureDesc::Checkerboard(even, odd) => Box::new(texture::Checkerboard::new(
                self.realize_texture(*even)?,
                self.realize_texture(*odd)?,
            )),
            desc::TextureDesc::Perlin => Box::new(texture::Perlin::default()),
            desc::TextureDesc::Image(path) => {
                let original = path.to_string_lossy().to_string();
                let mut dir = self.scene_path.clone();
                dir.pop();
                let adjusted_path = dir.join(path);
                let img = texture::Image::new(&adjusted_path).context(format!(
                    "Adjusted original path {} to {}",
                    original,
                    adjusted_path.to_string_lossy()
                ))?;
                Box::new(img)
            }
        })
    }

    pub(crate) fn eval_vec3(
        &self,
        (e1, e2, e3): (desc::Value, desc::Value, desc::Value),
    ) -> Result<Vec3> {
        Ok(Vec3::new(e1.eval(self)?, e2.eval(self)?, e3.eval(self)?))
    }
}
