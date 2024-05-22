use bevy_ecs::{bundle, entity, prelude::*};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Alive;
#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}

fn print(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {
        println!("{} {} {}", position.x, position.y, velocity.x);
    }
}

fn system_changed(query: Query<&Position, Changed<Velocity>>) {
    for position in &query {
    }
}

fn system_added(query: Query<&Position, Added<Velocity>>) {
    for position in &query {
    }
}

fn main() {
    let mut world = World::new();

    let bundle = (Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 });
    let mut entity: EntityWorldMut = world.spawn(bundle);
    entity.insert(Alive);

    let mut schedule = Schedule::default();

    schedule.add_systems(movement).add_systems(print);

    loop {
        schedule.run(&mut world);
    }
}
