use anyhow::Result;

use crate::mesh::Vec4;

#[derive(Copy, Clone, Debug)]
pub struct Light {
    //intensity放在dir的w分量
    pub direction: Vec4,
    //pub intensity: f32,
    pub color: Vec4,
}

impl Light {
    pub unsafe fn new(
        direction: Vec4,
        color: Vec4,
        //intensity: f32
    ) -> Result<Self> {
        Ok(Self {
            direction,
            color,
            //intensity
        })
    }
}