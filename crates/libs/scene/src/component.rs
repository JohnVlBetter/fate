pub trait ComponentBase {
    fn id(&self) -> u32;
    fn set_id(&mut self, id: u32);
    fn get_pred() -> impl Fn(&&Component) -> bool;
}

#[derive(PartialEq, PartialOrd, Debug)]
pub enum Component {
    MeshRenderer(MeshRenderer),
    Transform(Transform),
    Camera(Camera),
    Light(Light),
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct MeshRenderer {
    pub id: u32,
    pub node_id: u32,
    pub mesh: String,
}

impl ComponentBase for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_pred() -> impl Fn(&&Component) -> bool {
        move |comp| {
            if let Component::MeshRenderer(_) = comp {
                true
            } else {
                false
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct Transform {
    pub id: u32,
    pub node_id: u32,
    pub matrix: String,
}

impl ComponentBase for Transform {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_pred() -> impl Fn(&&Component) -> bool {
        move |comp| {
            if let Component::Transform(_) = comp {
                true
            } else {
                false
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct Camera {
    pub id: u32,
    pub node_id: u32,
    pub view: String,
}

impl ComponentBase for Camera {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_pred() -> impl Fn(&&Component) -> bool {
        move |comp| {
            if let Component::Camera(_) = comp {
                true
            } else {
                false
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug)]
pub struct Light {
    pub id: u32,
    pub node_id: u32,
    pub color: String,
}

impl ComponentBase for Light {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_pred() -> impl Fn(&&Component) -> bool {
        move |comp| {
            if let Component::Light(_) = comp {
                true
            } else {
                false
            }
        }
    }
}