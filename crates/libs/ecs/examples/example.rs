use bevy_ecs::{bundle, entity, prelude::*};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Alive;
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Resource)]
struct GlobalPosMinMax {
    min: f32,
    max: f32,
}

impl FromWorld for GlobalPosMinMax {
    fn from_world(world: &mut World) -> Self {
        GlobalPosMinMax {
            min: 0.0,
            max: 10000.0,
        }
    }
}

fn movement(
    mut query: Query<(&Alive, &mut Position, &Velocity)>,
    mut goals: ResMut<GlobalPosMinMax>,
) {
    for (_, mut position, velocity) in &mut query {
        if ((position.x + velocity.x) >= goals.min && (position.x + velocity.x) <= goals.max) {
            position.x += velocity.x;
            position.y += velocity.y;
        }
    }
}

fn death(mut query: Query<(Entity, &Alive, &mut Position)>, mut commands: Commands) {
    for (entity, _, mut position) in &mut query {
        if (position.x >= 8000.0) {
            commands.entity(entity).remove::<Alive>();
        }
    }
}

fn print(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        println!("{} {} {}", position.x, position.y, velocity.x);
    }
}

fn main() {
    let mut world = World::new();

    let bundle1 = (
        Alive,
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.0 },
    );
    let bundle2 = (
        Alive,
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 2.0, y: 0.0 },
    );
    world.spawn(bundle1);
    world.spawn(bundle2);

    world.init_resource::<GlobalPosMinMax>();

    let mut schedule = Schedule::default();

    schedule.add_systems(movement).add_systems(print).add_systems(death);

    loop {
        schedule.run(&mut world);
    }
}
