use anyhow::Result;

use crate::model::Vec3;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vec3,
    pub euler: Vec3,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, euler: Vec3, scale: Vec3) -> Result<Self> {
        Ok(Self {
            position,
            euler,
            scale,
        })
    }
}