pub trait Component {
    fn id(&self) -> u32;
    fn set_id(&mut self, id: u32);
}

pub struct MeshRenderer {
    pub id: u32,
    pub node_id: u32,
    pub mesh: String,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}
