use crate::{camera::Camera, hittable::Hittable};
use serde::{Deserialize, Serialize};

pub struct Scene<H: Hittable> {
    pub world: H,
    pub camera: Camera,
    pub image: ImageConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub samples_per_pixel: u32,
    pub max_depth: u32,
}
