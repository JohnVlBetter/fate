use bevy_ecs::world::World;

pub struct Scene {
    pub world: World,
}

impl Scene {
    pub fn new(world: World) -> Self {
        Self { world }
    }
}
