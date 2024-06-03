use bevy_ecs::world::World;
use scene::scene::Scene;
use scene::hierarchy::*;

fn main() {
    let world = World::new();
    let scene = Scene::new(world);
}
