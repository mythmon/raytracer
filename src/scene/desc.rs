use crate::{
    config,
    geom::{Point3, Vec3},
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
    pub(crate) materials: HashMap<String, Material>,
    pub(crate) objects: Vec<Hittable>,
    pub(crate) camera: Camera,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Object")]
pub(crate) enum Hittable {
    Sphere {
        center: (Value, Value, Value),
        radius: Value,
        material: Material,
    },
    MovingSphere {
        center: ((Value, Value, Value), (Value, Value, Value)),
        time: (Value, Value),
        radius: Value,
        material: Material,
    },
    AARect {
        center: (Value, Value, Value),
        width: Value,
        height: Value,
        axis: Axis,
        material: Material,
    },
    Cuboid {
        center: Option<(Value, Value, Value)>,
        size: (Value, Value, Value),
        material: Material,
    },
    Pattern {
        var: String,
        range: Vec<i32>,
        object: Box<Hittable>,
    },
    Translate {
        offset: (Value, Value, Value),
        hittable: Box<Hittable>,
    },
    RotateY {
        angle: Value,
        hittable: Box<Hittable>
    },
    ConstantMedium {
        boundary: Box<Hittable>,
        density: Value,
        texture: TextureDesc,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub(crate) enum Value {
    Var(String),
    Number(f64),
    BinOp(BinOp, Box<Value>, Box<Value>),
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub(crate) enum BinOp {
    Add,
    Mult,
    Rand,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "Material")]
pub(crate) enum Material {
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
    RandomChoice(Vec<Material>),
    RandomChoiceWeighted(Vec<(f64, Box<Material>)>),
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
pub(crate) struct Camera {
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
pub(crate) enum Axis {
    X,
    Y,
    Z,
}

impl From<Axis> for crate::geom::Axis {
    fn from(value: Axis) -> Self {
        match value {
            Axis::X => crate::geom::Axis::X,
            Axis::Y => crate::geom::Axis::Y,
            Axis::Z => crate::geom::Axis::Z,
        }
    }
}
