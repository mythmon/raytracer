use crate::{
    config,
    geom::{Axis, Point3, Vec3},
    scene::SceneLoader,
};
use anyhow::{anyhow, Result};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Scene")]
pub(crate) struct SceneDesc {
    pub(crate) materials: HashMap<String, MaterialDesc>,
    pub(crate) objects: Vec<HittableDesc>,
    pub(crate) camera: CameraDesc,
    pub(crate) image: config::Image,
    pub(crate) background: Option<(Value, Value, Value)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum TextureDesc {
    Solid(f64, f64, f64),
    Checkerboard(Box<TextureDesc>, Box<TextureDesc>),
    Perlin,
    Image(PathBuf),
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl From<Vec3> for (Value, Value, Value) {
    fn from(Vec3(x, y, z): Vec3) -> Self {
        (Value::Number(x), Value::Number(y), Value::Number(z))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Object")]
pub(crate) enum HittableDesc {
    Sphere {
        center: (Value, Value, Value),
        radius: Value,
        material: MaterialDesc,
    },
    MovingSphere {
        center: ((Value, Value, Value), (Value, Value, Value)),
        time: (Value, Value),
        radius: Value,
        material: MaterialDesc,
    },
    Pattern {
        var: String,
        range: Vec<i32>,
        object: Box<HittableDesc>,
    },
    AARect {
        center: (Value, Value, Value),
        width: Value,
        height: Value,
        axis: AxisDesc,
        material: MaterialDesc,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum Value {
    Var(String),
    Number(f64),
    BinOp(BinOp, Box<Value>, Box<Value>),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename = "lowercase")]
pub(crate) enum BinOp {
    Add,
    Mult,
    Rand,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Material")]
pub(crate) enum MaterialDesc {
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
    DiffuseLight {
        color: TextureDesc,
    },
    RandomChoice(Vec<MaterialDesc>),
    RandomChoiceWeighted(Vec<(f64, Box<MaterialDesc>)>),
}

impl Value {
    pub(crate) fn eval(&self, loader: &SceneLoader) -> Result<f64> {
        match self {
            Value::Var(var) => loader
                .pattern_vars
                .get(var)
                .ok_or_else(|| anyhow!("Variable {} not found", var))
                .map(|n| *n as f64),
            Value::Number(n) => Ok(*n),
            Value::BinOp(op, a, b) => {
                let a = a.eval(loader)?;
                let b = b.eval(loader)?;
                match op {
                    BinOp::Rand => {
                        let mut rng = thread_rng();
                        Ok(rng.gen_range(a..b))
                    }
                    BinOp::Add => Ok(a + b),
                    BinOp::Mult => Ok(a * b),
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "Camera")]
pub(crate) struct CameraDesc {
    pub(crate) look_from: Point3,
    pub(crate) look_at: Option<Point3>,
    pub(crate) v_up: Option<Vec3>,
    pub(crate) vertical_fov: f64,
    pub(crate) aperture: f64,
    pub(crate) focus_distance: Option<f64>,
    pub(crate) shutter_time: Option<(f64, f64)>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename = "Axis")]
pub(crate) enum AxisDesc {
    X,
    Y,
    Z,
}

impl From<AxisDesc> for Axis {
    fn from(value: AxisDesc) -> Self {
        match value {
            AxisDesc::X => Axis::X,
            AxisDesc::Y => Axis::Y,
            AxisDesc::Z => Axis::Z,
        }
    }
}
