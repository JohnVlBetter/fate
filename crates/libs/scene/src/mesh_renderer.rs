use crate::component::Component;
use std::any::Any;

pub struct MeshRenderer {
    pub id: u32,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        "MeshRenderer"
    }

    fn start(&mut self) {}

    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        MeshRenderer { id: 0 }
    }
}
