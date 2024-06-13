pub trait Component {
    fn id(&self) -> u32;
}

pub struct MeshRenderer {
    pub(crate) id: u32,
    pub(crate) node_id: u32,
    pub(crate) mesh: String,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }
}
