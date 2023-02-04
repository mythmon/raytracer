use crate::{
    camera::Camera,
    config::{self, Scene},
    geom::{Color, Point3, Vec3},
    hittable::{self, BvhNode, Hittable},
    material,
    texture::{Checkerboard, SolidColor, Texture, Perlin},
};
use anyhow::{anyhow, Context, Result};
use rand::{prelude::Distribution, thread_rng, Rng};
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, sync::Arc};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Scene")]
struct SceneDesc {
    materials: HashMap<String, MaterialDesc>,
    objects: Vec<HittableDesc>,
    camera: CameraDesc,
    image: config::ImageConfig,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
enum TextureDesc {
    Solid(f64, f64, f64),
    Checkerboard(Box<TextureDesc>, Box<TextureDesc>),
    Perlin,
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl From<Vec3> for (Value, Value, Value) {
    fn from(Vec3(x, y, z): Vec3) -> Self {
        (
            Value::Number(x),
            Value::Number(y),
            Value::Number(z),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Object")]
enum HittableDesc {
    Sphere {
        center: (Value, Value, Value),
        radius: Value,
        material: MaterialDesc,
    },
    MovingSphere {
        center: (
            (Value, Value, Value),
            (Value, Value, Value),
        ),
        time: (Value, Value),
        radius: Value,
        material: MaterialDesc,
    },
    Pattern {
        var: String,
        range: Vec<i32>,
        object: Box<HittableDesc>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Value {
    Var(String),
    Number(f64),
    BinOp(String, Box<Value>, Box<Value>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Material")]
enum MaterialDesc {
    Shared(String),
    Lambertian {
        albedo: TextureDesc,
    },
    Metal {
        albedo: (Value, Value, Value),
        fuzziness: Value,
    },
    Dielectric {
        index_of_refraction: Value,
    },
    RandomChoice(Vec<Box<MaterialDesc>>),
    RandomChoiceWeighted(Vec<(f64, Box<MaterialDesc>)>),
}

impl Value {
    fn eval(&self, context: &PatternContext) -> Result<f64> {
        match self {
            Value::Var(var) => context
                .vars
                .get(var)
                .ok_or_else(|| anyhow!("Variable {} not found", var))
                .map(|n| *n as f64),
            Value::Number(n) => Ok(*n),
            Value::BinOp(op, a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                match op.as_str() {
                    "rand" => {
                        let mut rng = thread_rng();
                        Ok(rng.gen_range(a..b))
                    }
                    "add" => Ok(a + b),
                    "mult" => Ok(a * b),
                    _ => Err(anyhow!("Unknown operation {}", op)),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Camera")]
struct CameraDesc {
    look_from: Point3,
    look_at: Option<Point3>,
    v_up: Option<Vec3>,
    vertical_fov: f64,
    aperture: f64,
    focus_distance: Option<f64>,
    shutter_time: Option<(f64, f64)>,
}

pub fn load_scene(path: &Path) -> Result<Scene<BvhNode>> {
    let f = std::fs::File::open(path).context("opening scene file")?;

    let ron_options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
    let scene_desc: SceneDesc = ron_options.from_reader(f).context("loading scene file")?;
    let context = PatternContext::default();

    let mut materials: HashMap<String, Arc<dyn material::Material>> = HashMap::new();
    for (key, desc) in scene_desc.materials.into_iter() {
        let material = realize_material(&materials, desc, context.clone())?;
        materials.insert(key, material);
    }

    let mut hittables: Vec<Box<dyn Hittable>> = vec![];
    for desc in scene_desc.objects.into_iter() {
        match desc {
            HittableDesc::Sphere {
                material,
                center,
                radius,
            } => {
                let material = realize_material(&materials, material, context.clone())?;
                // let material = match material {
                //     MaterialDesc::Shared(name) => materials
                //         .get(&name)
                //         .ok_or_else(|| anyhow!("Material {} not defined", name))?
                //         .clone(),
                //     MaterialDesc::Lambertian { albedo } => Arc::new(material::Lambertian {
                //         albedo: realize_texture(albedo),
                //     }),
                //     MaterialDesc::Dielectric {
                //         index_of_refraction,
                //     } => Arc::new(material::Dielectric {
                //         index_of_refraction,
                //     }),
                //     MaterialDesc::Metal { albedo, fuzziness } => {
                //         Arc::new(material::Metal { albedo, fuzziness })
                //     }
                // };
                let center = Point3::new(
                    center.0.eval(&context)?,
                    center.1.eval(&context)?,
                    center.2.eval(&context)?,
                );

                hittables.push(Box::new(hittable::Sphere {
                    center,
                    radius: radius.eval(&context)?,
                    material,
                }))
            }
            HittableDesc::MovingSphere { center, time, radius, material } => {
                hittables.push(Box::new(hittable::MovingSphere {
                    center: (eval_vec3(&context, center.0)?)..(eval_vec3(&context, center.1)?),
                    time: (time.0.eval(&context)?)..(time.1.eval(&context)?),
                    radius: radius.eval(&context)?,
                    material: realize_material(&materials, material, context.clone())?,
                }))
            }
            HittableDesc::Pattern { var, range, object } => {
                realize_pattern(&mut hittables, var, &range[..], &*object, None, &materials)?;
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
    })
}

#[derive(Clone, Default)]
struct PatternContext {
    vars: HashMap<String, i32>,
}

fn realize_pattern(
    hittables: &mut Vec<Box<dyn Hittable>>,
    var: String,
    range: &[i32],
    object: &HittableDesc,
    context: Option<PatternContext>,
    materials: &HashMap<String, Arc<dyn material::Material>>,
) -> Result<()> {
    let mut context = context.unwrap_or_default();

    let range = match range {
        &[end] => (0..end).step_by(1),
        &[start, end] => (start..end).step_by(1),
        &[start, end, step] => (start..end).step_by(step as usize),
        unexpected => {
            anyhow::bail!("Unexpected format for range: {:?}", unexpected);
        }
    };

    for val in range {
        context.vars.insert(var.to_string(), val);
        match *object {
            HittableDesc::Sphere {
                ref center,
                ref radius,
                ref material,
            } => hittables.push(Box::new(hittable::Sphere {
                center: Vec3::new(
                    center.0.eval(&context)?,
                    center.1.eval(&context)?,
                    center.2.eval(&context)?,
                ),
                radius: radius.eval(&context)?,
                material: realize_material(materials, (*material).clone(), context.clone())?,
            })),

            HittableDesc::MovingSphere {
                ref center,
                ref time,
                ref radius,
                ref material,
            } => {
                let c1 = Vec3::new(
                    center.0 .0.eval(&context)?,
                    center.0 .1.eval(&context)?,
                    center.0 .2.eval(&context)?,
                );
                let c2 = Vec3::new(
                    center.1 .0.eval(&context)?,
                    center.1 .1.eval(&context)?,
                    center.1 .2.eval(&context)?,
                );
                hittables.push(Box::new(hittable::MovingSphere {
                    center: c1..c2,
                    time: (time.0.eval(&context)?)..(time.1.eval(&context)?),
                    radius: radius.eval(&context)?,
                    material: realize_material(materials, (*material).clone(), context.clone())?,
                }))
            }

            HittableDesc::Pattern {
                ref var,
                ref range,
                ref object,
            } => {
                realize_pattern(
                    hittables,
                    var.clone(),
                    range,
                    object,
                    Some(context.clone()),
                    materials,
                )?;
            }
        }
    }

    Ok(())
}

fn realize_material<D: Into<MaterialDesc>>(
    cache: &HashMap<String, Arc<dyn material::Material>>,
    desc: D,
    context: PatternContext,
) -> Result<Arc<dyn material::Material>> {
    let desc: MaterialDesc = desc.into();
    Ok(match desc {
        MaterialDesc::Shared(ref name) => cache
            .get(name)
            .ok_or_else(|| anyhow!("Material {} not defined", name))?
            .clone(),
        MaterialDesc::Lambertian { albedo } => {
            Arc::new(material::Lambertian {albedo: realize_texture(albedo)})
        }
        MaterialDesc::Dielectric {
            index_of_refraction,
        } => Arc::new(material::Dielectric {
            index_of_refraction: index_of_refraction
                .eval(&context)
                .context("evaluating index_of_refraction")?,
        }),
        MaterialDesc::Metal { albedo, fuzziness } => Arc::new(material::Metal {
            albedo: Color::new(
                albedo.0.eval(&context).context("evaluating albedo r")?,
                albedo.1.eval(&context).context("evaluating albedo g")?,
                albedo.2.eval(&context).context("evaluating albedo b")?,
            ),
            fuzziness: fuzziness.eval(&context).context("evaluating fuzziness")?,
        }),
        MaterialDesc::RandomChoice(options) => {
            let mut rng = thread_rng();
            let idx = rng.gen_range(0..options.len());
            realize_material(cache, (*options[idx]).clone(), context.clone())?
        }
        MaterialDesc::RandomChoiceWeighted(options) => {
            let dist = rand::distributions::WeightedIndex::new(options.iter().map(|c| c.0))
                .context("generating weighted distribution")?;
            let mut rng = thread_rng();
            let idx = dist.sample(&mut rng);
            realize_material(cache, (*options[idx].1).clone(), context.clone())?
        }
    })
}

fn realize_texture(desc: TextureDesc) -> Box<dyn Texture> {
    match desc {
        TextureDesc::Solid(r, g, b) => Box::new(SolidColor(Color::new(r, g, b))),
        TextureDesc::Checkerboard(even, odd) => Box::new(Checkerboard::new(
            realize_texture(*even),
            realize_texture(*odd),
        )),
        TextureDesc::Perlin => Box::new(Perlin::default())
    }
}

fn eval_vec3(context: &PatternContext, (e1, e2, e3): (Value, Value, Value)) -> Result<Vec3> {
    Ok(Vec3::new(e1.eval(context)?, e2.eval(context)?, e3.eval(context)?))
}