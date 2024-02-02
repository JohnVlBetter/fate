use anyhow::Result;

#[derive(Copy, Clone, Debug)]
pub struct Light {
}

impl Light {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}