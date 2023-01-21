use serde::{Serialize, Deserialize};

use crate::{hittable::HittableList, camera::Camera};

pub struct Scene {
    pub world: HittableList,
    pub camera: Camera,
    pub image: ImageConfig
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
}