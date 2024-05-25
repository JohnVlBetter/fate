use bevy_ecs::{bundle, entity, prelude::*};

pub struct Application {
    world: World,
    schedule: Schedule,
}

//gltf loader
//scene
//mesh renderer


fn main() {
    let mut world = World::new();

    let mut schedule = Schedule::default();
    //world.run_schedule(label)
    loop {
        schedule.run(&mut world);
    }
}
