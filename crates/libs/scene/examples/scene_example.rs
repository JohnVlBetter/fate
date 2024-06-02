use bevy_ecs::world::World;
use scene::scene::Scene;

fn main() {
    let world = World::new();
    let scene = Scene::new(world);
}
