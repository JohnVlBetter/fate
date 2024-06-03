use bevy_ecs::world::World;
use scene::hierarchy::*;
use scene::scene::Scene;

fn main() {
    let world = World::new();
    let scene = Scene::new(world);
}
mod tests {
    use super::{BuildChildren, BuildWorldChildren, HierarchyEvent};
    use scene::children::Children;
    use smallvec::{smallvec, SmallVec};
    use HierarchyEvent::ChildAdded;
    use HierarchyEvent::ChildMoved;
    use HierarchyEvent::ChildRemoved;

    use bevy_ecs::{
        component::Component,
        entity::Entity,
        event::Events,
        system::{CommandQueue, Commands},
        world::World,
    };

    #[derive(Component)]
    struct C(u32);

    #[test]
    fn children_removed_when_empty_world() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3)])
            .collect::<Vec<Entity>>();

        let parent1 = entities[0];
        let parent2 = entities[1];
        let child = entities[2];

        world.entity_mut(parent1).push_children(&[child]);

        world.entity_mut(parent2).push_children(&[child]);
        assert!(world.get::<Children>(parent1).is_none());

        world.entity_mut(parent1).insert_children(0, &[child]);
        assert!(world.get::<Children>(parent2).is_none());

        world.entity_mut(parent1).remove_children(&[child]);
        assert!(world.get::<Children>(parent1).is_none());
    }

    #[test]
    fn children_removed_when_empty_commands() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3)])
            .collect::<Vec<Entity>>();

        let parent1 = entities[0];
        let parent2 = entities[1];
        let child = entities[2];

        let mut queue = CommandQueue::default();

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent1).push_children(&[child]);
            queue.apply(&mut world);
        }

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent2).push_children(&[child]);
            queue.apply(&mut world);
        }
        assert!(world.get::<Children>(parent1).is_none());

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent1).insert_children(0, &[child]);
            queue.apply(&mut world);
        }
        assert!(world.get::<Children>(parent2).is_none());

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent2).add_child(child);
            queue.apply(&mut world);
        }
        assert!(world.get::<Children>(parent1).is_none());

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent2).remove_children(&[child]);
            queue.apply(&mut world);
        }
        assert!(world.get::<Children>(parent2).is_none());
    }

    #[test]
    fn regression_push_children_same_archetype() {
        let mut world = World::new();
        let child = world.spawn_empty().id();
        world.spawn_empty().push_children(&[child]);
    }

    #[test]
    fn push_children_idempotent() {
        let mut world = World::new();
        let child = world.spawn_empty().id();
        let parent = world
            .spawn_empty()
            .push_children(&[child])
            .push_children(&[child])
            .id();

        let mut query = world.query::<&Children>();
        let children = query.get(&world, parent).unwrap();
        assert_eq!(**children, [child]);
    }
}
