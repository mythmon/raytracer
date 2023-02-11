use crate::{
    camera::Camera,
    config::Scene,
    geom::{Color, Vec3},
    hittable::{self, AxisAlignedRect, BvhNode, Cuboid, Hittable, RotateY, Translate, ConstantMedium},
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

#[derive(Default)]
struct HittableAccum(Vec<Box<dyn Hittable>>);

impl HittableAccum {
    fn add<H: Hittable + 'static>(&mut self, h: H) {
        self.0.push(Box::new(h));
    }

    fn add_many<I: Iterator<Item = Box<dyn Hittable>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

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

        for (key, desc) in scene_desc.materials {
            let material = self.realize_material(desc)?;
            self.materials.insert(key, material);
        }

        let mut hittables = HittableAccum::default();
        for desc in scene_desc.objects {
            self.realize_hittable(desc, &mut hittables)?;
        }

        let mut camera_builder = Camera::build()
            .look_from(scene_desc.camera.look_from)
            .aspect_ratio(scene_desc.image.width as f64 / scene_desc.image.height as f64)
            .vertical_fov(scene_desc.camera.vertical_fov)
            .aperture(scene_desc.camera.aperture);

        if let Some(look_at) = scene_desc.camera.look_at {
            camera_builder = camera_builder.look_at(look_at);
        }
        if let Some(v_up) = scene_desc.camera.v_up {
            camera_builder = camera_builder.v_up(v_up);
        }
        if let Some(focus_distance) = scene_desc.camera.focus_distance {
            camera_builder = camera_builder.focus_dist(focus_distance);
        }
        if let Some((start, end)) = scene_desc.camera.shutter_time {
            camera_builder = camera_builder.shutter_time(start..end);
        }

        let camera = camera_builder.done()?;

        Ok(Scene {
            world: BvhNode::new(camera.shutter_time.clone(), hittables.0),
            camera,
            image: scene_desc.image,
            background: scene_desc
                .background
                .map_or_else(|| Ok(Color::black()), |v| self.eval_vec3(v))?,
        })
    }

    fn realize_hittable(
        &mut self,
        hittable: desc::Hittable,
        hittables: &mut HittableAccum,
    ) -> Result<()> {
        match hittable {
            desc::Hittable::Sphere {
                material,
                center,
                radius,
            } => hittables.add(hittable::Sphere {
                center: self.eval_vec3(center)?,
                radius: radius.eval(self)?,
                material: self.realize_material(material)?,
            }),

            desc::Hittable::MovingSphere {
                center,
                time,
                radius,
                material,
            } => hittables.add(hittable::MovingSphere {
                center: (self.eval_vec3(center.0)?)..(self.eval_vec3(center.1)?),
                time: (time.0.eval(self)?)..(time.1.eval(self)?),
                radius: radius.eval(self)?,
                material: self.realize_material(material)?,
            }),

            desc::Hittable::AARect {
                center,
                width,
                height,
                axis,
                material,
            } => hittables.add(AxisAlignedRect {
                center: self.eval_vec3(center)?,
                width: width.eval(self)?,
                height: height.eval(self)?,
                axis: axis.into(),
                material: self.realize_material(material)?,
            }),

            desc::Hittable::Cuboid {
                center,
                size,
                material,
            } => hittables.add(Cuboid::new(
                center.map_or_else(|| Ok(Vec3::default()), |c| self.eval_vec3(c))?,
                self.eval_vec3(size)?,
                &self.realize_material(material)?,
            )),

            desc::Hittable::Pattern { var, range, object } => {
                self.realize_pattern(&var, &range[..], &object, hittables)?;
            }

            desc::Hittable::Translate { offset, hittable } => {
                let offset = self.eval_vec3(offset)?;
                let mut inner = HittableAccum::default();
                self.realize_hittable(*hittable, &mut inner)?;
                hittables.add_many(inner.0.into_iter().map(|h| {
                    Box::new(Translate {
                        offset,
                        hittable: h,
                    }) as Box<dyn Hittable>
                }));
            }

            desc::Hittable::RotateY { angle, hittable } => {
                let theta = angle.eval(self)?.to_radians();
                let mut inner = HittableAccum::default();
                self.realize_hittable(*hittable, &mut inner)?;
                hittables.add_many(
                    inner
                        .0
                        .into_iter()
                        .map(|h| Box::new(RotateY::new(h, theta)) as Box<dyn Hittable>),
                );
            }

            desc::Hittable::ConstantMedium { boundary, density, texture: color } => {
                let mut inner = HittableAccum::default();
                let texture = self.realize_texture(color)?;
                let density = density.eval(self)?;
                self.realize_hittable(*boundary, &mut inner)?;
                hittables.add_many(
                    inner
                        .0
                        .into_iter()
                        .map(|h| Box::new(ConstantMedium::new(h, density, texture.clone())) as Box<dyn Hittable>),
                );
            }
        };
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn realize_pattern(
        &mut self,
        var: &str,
        range: &[i32],
        object: &desc::Hittable,
        hittables: &mut HittableAccum,
    ) -> Result<()> {
        let range = match range {
            &[end] => (0..end).step_by(1),
            &[start, end] => (start..end).step_by(1),
            &[start, end, step] => (start..end).step_by(step.unsigned_abs() as usize),
            unexpected => {
                anyhow::bail!("Unexpected format for range: {:?}", unexpected);
            }
        };

        for val in range {
            self.pattern_vars.insert(var.to_string(), val);
            match object {
                desc::Hittable::Sphere {
                    ref center,
                    ref radius,
                    ref material,
                } => hittables.add(hittable::Sphere {
                    center: self.eval_vec3(center.clone())?,
                    radius: radius.eval(self)?,
                    material: self.realize_material((*material).clone())?,
                }),

                desc::Hittable::MovingSphere {
                    ref center,
                    ref time,
                    ref radius,
                    ref material,
                } => {
                    let c1 = self.eval_vec3(center.0.clone())?;
                    let c2 = self.eval_vec3(center.1.clone())?;
                    hittables.add(hittable::MovingSphere {
                        center: c1..c2,
                        time: (time.0.eval(self)?)..(time.1.eval(self)?),
                        radius: radius.eval(self)?,
                        material: self.realize_material((*material).clone())?,
                    });
                }

                desc::Hittable::AARect {
                    ref center,
                    ref width,
                    ref height,
                    ref axis,
                    ref material,
                } => hittables.add(AxisAlignedRect {
                    center: self.eval_vec3(center.clone())?,
                    width: width.eval(self)?,
                    height: height.eval(self)?,
                    axis: (*axis).into(),
                    material: self.realize_material((*material).clone())?,
                }),

                desc::Hittable::Cuboid {
                    ref center,
                    ref size,
                    ref material,
                } => hittables.add(Cuboid::new(
                    center
                        .clone()
                        .map_or_else(|| Ok(Vec3::default()), |c| self.eval_vec3(c))?,
                    self.eval_vec3(size.clone())?,
                    &self.realize_material((*material).clone())?,
                )),

                desc::Hittable::Pattern {
                    ref var,
                    ref range,
                    ref object,
                } => {
                    self.realize_pattern(var, range, object, hittables)?;
                }

                desc::Hittable::Translate { offset, hittable } => {
                    let offset = self.eval_vec3(offset.clone())?;
                    let mut inner = HittableAccum::default();
                    self.realize_hittable((**hittable).clone(), &mut inner)?;
                    hittables.add_many(inner.0.into_iter().map(|h| {
                        Box::new(Translate {
                            offset,
                            hittable: h,
                        }) as Box<dyn Hittable>
                    }));
                }

                desc::Hittable::RotateY { angle, hittable } => {
                    let theta = angle.eval(self)?.to_radians();
                    let mut inner = HittableAccum::default();
                    self.realize_hittable((**hittable).clone(), &mut inner)?;
                    hittables.add_many(
                        inner
                            .0
                            .into_iter()
                            .map(|h| Box::new(RotateY::new(h, theta)) as Box<dyn Hittable>),
                    );
                }

                desc::Hittable::ConstantMedium { boundary, density, texture: color } => {
                    let mut inner = HittableAccum::default();
                    let texture = self.realize_texture((*color).clone())?;
                    let density = density.eval(self)?;
                    self.realize_hittable((**boundary).clone(), &mut inner)?;
                    hittables.add_many(
                        inner
                            .0
                            .into_iter()
                            .map(|h| Box::new(ConstantMedium::new(h, density, texture.clone())) as Box<dyn Hittable>),
                    );
                }
            }
        }

        Ok(())
    }

    pub(crate) fn realize_material<D: Into<desc::Material>>(
        &self,
        desc: D,
    ) -> Result<Arc<dyn material::Material>> {
        let desc: desc::Material = desc.into();
        Ok(match desc {
            desc::Material::Shared(ref name) => self
                .materials
                .get(name)
                .ok_or_else(|| anyhow!("Material {} not defined", name))?
                .clone(),
            desc::Material::Lambertian { albedo } => Arc::new(material::Lambertian {
                albedo: self.realize_texture(albedo)?,
            }),
            desc::Material::Dielectric {
                index_of_refraction,
            } => Arc::new(material::Dielectric {
                index_of_refraction: index_of_refraction
                    .eval(self)
                    .context("evaluating index_of_refraction")?,
            }),
            desc::Material::Metal { albedo, fuzziness } => Arc::new(material::Metal {
                albedo: self.eval_vec3(albedo)?,
                fuzziness: fuzziness.eval(self).context("evaluating fuzziness")?,
            }),
            desc::Material::DiffuseLight { color } => Arc::new(material::DiffuseLight {
                texture: self.realize_texture(color)?,
            }),
            desc::Material::RandomChoice(options) => {
                let mut rng = thread_rng();
                let idx = rng.gen_range(0..options.len());
                self.realize_material((options[idx]).clone())?
            }
            desc::Material::RandomChoiceWeighted(options) => {
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
            desc::TextureDesc::Perlin => Box::<texture::Perlin>::default(),
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
