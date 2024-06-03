use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    event::Event,
    prelude::Events,
    system::{Command, Commands, EntityCommands},
    world::{EntityWorldMut, World},
};
use smallvec::SmallVec;

use crate::children::Children;
use crate::parent::Parent;

#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub enum HierarchyEvent {
    ChildAdded {
        child: Entity,
        parent: Entity,
    },
    ChildRemoved {
        child: Entity,
        parent: Entity,
    },
    ChildMoved {
        child: Entity,
        previous_parent: Entity,
        new_parent: Entity,
    },
}

fn push_events(world: &mut World, events: impl IntoIterator<Item = HierarchyEvent>) {
    if let Some(mut moved) = world.get_resource_mut::<Events<HierarchyEvent>>() {
        moved.extend(events);
    }
}

fn push_child_unchecked(world: &mut World, parent: Entity, child: Entity) {
    let mut parent = world.entity_mut(parent);
    if let Some(mut children) = parent.get_mut::<Children>() {
        children.0.push(child);
    } else {
        parent.insert(Children(smallvec::smallvec![child]));
    }
}

fn update_parent(world: &mut World, child: Entity, new_parent: Entity) -> Option<Entity> {
    let mut child = world.entity_mut(child);
    if let Some(mut parent) = child.get_mut::<Parent>() {
        let previous = parent.0;
        *parent = Parent(new_parent);
        Some(previous)
    } else {
        child.insert(Parent(new_parent));
        None
    }
}

fn remove_from_children(world: &mut World, parent: Entity, child: Entity) {
    let Some(mut parent) = world.get_entity_mut(parent) else {
        return;
    };
    let Some(mut children) = parent.get_mut::<Children>() else {
        return;
    };
    children.0.retain(|x| *x != child);
    if children.0.is_empty() {
        parent.remove::<Children>();
    }
}

fn update_old_parent(world: &mut World, child: Entity, parent: Entity) {
    let previous = update_parent(world, child, parent);
    if let Some(previous_parent) = previous {
        if previous_parent == parent {
            return;
        }
        remove_from_children(world, previous_parent, child);

        push_events(
            world,
            [HierarchyEvent::ChildMoved {
                child,
                previous_parent,
                new_parent: parent,
            }],
        );
    } else {
        push_events(world, [HierarchyEvent::ChildAdded { child, parent }]);
    }
}

fn update_old_parents(world: &mut World, parent: Entity, children: &[Entity]) {
    let mut events: SmallVec<[HierarchyEvent; 8]> = SmallVec::with_capacity(children.len());
    for &child in children {
        if let Some(previous) = update_parent(world, child, parent) {
            if parent == previous {
                continue;
            }

            remove_from_children(world, previous, child);
            events.push(HierarchyEvent::ChildMoved {
                child,
                previous_parent: previous,
                new_parent: parent,
            });
        } else {
            events.push(HierarchyEvent::ChildAdded { child, parent });
        }
    }
    push_events(world, events);
}

fn remove_children(parent: Entity, children: &[Entity], world: &mut World) {
    let mut events: SmallVec<[HierarchyEvent; 8]> = SmallVec::new();
    if let Some(parent_children) = world.get::<Children>(parent) {
        for &child in children {
            if parent_children.0.contains(&child) {
                events.push(HierarchyEvent::ChildRemoved { child, parent });
            }
        }
    } else {
        return;
    }
    for event in &events {
        if let &HierarchyEvent::ChildRemoved { child, .. } = event {
            world.entity_mut(child).remove::<Parent>();
        }
    }
    push_events(world, events);

    let mut parent = world.entity_mut(parent);
    if let Some(mut parent_children) = parent.get_mut::<Children>() {
        parent_children
            .0
            .retain(|parent_child| !children.contains(parent_child));

        if parent_children.0.is_empty() {
            parent.remove::<Children>();
        }
    }
}

fn clear_children(parent: Entity, world: &mut World) {
    if let Some(children) = world.entity_mut(parent).take::<Children>() {
        for &child in &children.0 {
            world.entity_mut(child).remove::<Parent>();
        }
    }
}

#[derive(Debug)]
pub struct AddChild {
    pub parent: Entity,
    pub child: Entity,
}

impl Command for AddChild {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.parent).add_child(self.child);
    }
}

#[derive(Debug)]
pub struct InsertChildren {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
    index: usize,
}

impl Command for InsertChildren {
    fn apply(self, world: &mut World) {
        world
            .entity_mut(self.parent)
            .insert_children(self.index, &self.children);
    }
}

#[derive(Debug)]
pub struct PushChildren {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl Command for PushChildren {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.parent).push_children(&self.children);
    }
}

pub struct RemoveChildren {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl Command for RemoveChildren {
    fn apply(self, world: &mut World) {
        remove_children(self.parent, &self.children, world);
    }
}

pub struct ClearChildren {
    parent: Entity,
}

impl Command for ClearChildren {
    fn apply(self, world: &mut World) {
        clear_children(self.parent, world);
    }
}

pub struct ReplaceChildren {
    parent: Entity,
    children: SmallVec<[Entity; 8]>,
}

impl Command for ReplaceChildren {
    fn apply(self, world: &mut World) {
        clear_children(self.parent, world);
        world.entity_mut(self.parent).push_children(&self.children);
    }
}

pub struct RemoveParent {
    pub child: Entity,
}

impl Command for RemoveParent {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.child).remove_parent();
    }
}

pub struct ChildBuilder<'w, 's, 'a> {
    commands: &'a mut Commands<'w, 's>,
    push_children: PushChildren,
}

impl<'w, 's, 'a> ChildBuilder<'w, 's, 'a> {
    pub fn spawn(&mut self, bundle: impl Bundle) -> EntityCommands<'_> {
        let e = self.commands.spawn(bundle);
        self.push_children.children.push(e.id());
        e
    }

    pub fn spawn_empty(&mut self) -> EntityCommands<'_> {
        let e = self.commands.spawn_empty();
        self.push_children.children.push(e.id());
        e
    }

    pub fn parent_entity(&self) -> Entity {
        self.push_children.parent
    }

    pub fn add_command<C: Command + 'static>(&mut self, command: C) -> &mut Self {
        self.commands.add(command);
        self
    }
}

pub trait BuildChildren {
    fn with_children(&mut self, f: impl FnOnce(&mut ChildBuilder)) -> &mut Self;

    fn push_children(&mut self, children: &[Entity]) -> &mut Self;

    fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self;

    fn remove_children(&mut self, children: &[Entity]) -> &mut Self;

    fn add_child(&mut self, child: Entity) -> &mut Self;

    fn clear_children(&mut self) -> &mut Self;

    fn replace_children(&mut self, children: &[Entity]) -> &mut Self;

    fn set_parent(&mut self, parent: Entity) -> &mut Self;

    fn remove_parent(&mut self) -> &mut Self;
}

impl<'a> BuildChildren for EntityCommands<'a> {
    fn with_children(&mut self, spawn_children: impl FnOnce(&mut ChildBuilder)) -> &mut Self {
        let parent = self.id();
        let mut builder = ChildBuilder {
            commands: &mut self.commands(),
            push_children: PushChildren {
                children: SmallVec::default(),
                parent,
            },
        };

        spawn_children(&mut builder);
        let children = builder.push_children;
        if children.children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.commands().add(children);
        self
    }

    fn push_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        if children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.commands().add(PushChildren {
            children: SmallVec::from(children),
            parent,
        });
        self
    }

    fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        if children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.commands().add(InsertChildren {
            children: SmallVec::from(children),
            index,
            parent,
        });
        self
    }

    fn remove_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        self.commands().add(RemoveChildren {
            children: SmallVec::from(children),
            parent,
        });
        self
    }

    fn add_child(&mut self, child: Entity) -> &mut Self {
        let parent = self.id();
        if child == parent {
            panic!("子节点不能是自己");
        }
        self.commands().add(AddChild { child, parent });
        self
    }

    fn clear_children(&mut self) -> &mut Self {
        let parent = self.id();
        self.commands().add(ClearChildren { parent });
        self
    }

    fn replace_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        if children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.commands().add(ReplaceChildren {
            children: SmallVec::from(children),
            parent,
        });
        self
    }

    fn set_parent(&mut self, parent: Entity) -> &mut Self {
        let child = self.id();
        if child == parent {
            panic!("父节点不能是自己");
        }
        self.commands().add(AddChild { child, parent });
        self
    }

    fn remove_parent(&mut self) -> &mut Self {
        let child = self.id();
        self.commands().add(RemoveParent { child });
        self
    }
}

#[derive(Debug)]
pub struct WorldChildBuilder<'w> {
    world: &'w mut World,
    parent: Entity,
}

impl<'w> WorldChildBuilder<'w> {
    pub fn spawn(&mut self, bundle: impl Bundle + Send + Sync + 'static) -> EntityWorldMut<'_> {
        let entity = self.world.spawn((bundle, Parent(self.parent))).id();
        push_child_unchecked(self.world, self.parent, entity);
        push_events(
            self.world,
            [HierarchyEvent::ChildAdded {
                child: entity,
                parent: self.parent,
            }],
        );
        self.world.entity_mut(entity)
    }

    pub fn spawn_empty(&mut self) -> EntityWorldMut<'_> {
        let entity = self.world.spawn(Parent(self.parent)).id();
        push_child_unchecked(self.world, self.parent, entity);
        push_events(
            self.world,
            [HierarchyEvent::ChildAdded {
                child: entity,
                parent: self.parent,
            }],
        );
        self.world.entity_mut(entity)
    }

    pub fn parent_entity(&self) -> Entity {
        self.parent
    }
}

pub trait BuildWorldChildren {
    fn with_children(&mut self, spawn_children: impl FnOnce(&mut WorldChildBuilder)) -> &mut Self;

    fn add_child(&mut self, child: Entity) -> &mut Self;

    fn push_children(&mut self, children: &[Entity]) -> &mut Self;

    fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self;

    fn remove_children(&mut self, children: &[Entity]) -> &mut Self;

    fn set_parent(&mut self, parent: Entity) -> &mut Self;

    fn remove_parent(&mut self) -> &mut Self;

    fn clear_children(&mut self) -> &mut Self;

    fn replace_children(&mut self, children: &[Entity]) -> &mut Self;
}

impl<'w> BuildWorldChildren for EntityWorldMut<'w> {
    fn with_children(&mut self, spawn_children: impl FnOnce(&mut WorldChildBuilder)) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            spawn_children(&mut WorldChildBuilder { world, parent });
        });
        self
    }

    fn add_child(&mut self, child: Entity) -> &mut Self {
        let parent = self.id();
        if child == parent {
            panic!("子节点不能是自己");
        }
        self.world_scope(|world| {
            update_old_parent(world, child, parent);
        });
        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component.0.retain(|value| child != *value);
            children_component.0.push(child);
        } else {
            self.insert(Children::from_entities(&[child]));
        }
        self
    }

    fn push_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        if children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.world_scope(|world| {
            update_old_parents(world, parent, children);
        });
        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component
                .0
                .retain(|value| !children.contains(value));
            children_component.0.extend(children.iter().cloned());
        } else {
            self.insert(Children::from_entities(children));
        }
        self
    }

    fn insert_children(&mut self, index: usize, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        if children.contains(&parent) {
            panic!("子节点不能是自己");
        }
        self.world_scope(|world| {
            update_old_parents(world, parent, children);
        });
        if let Some(mut children_component) = self.get_mut::<Children>() {
            children_component
                .0
                .retain(|value| !children.contains(value));
            children_component.0.insert_from_slice(index, children);
        } else {
            self.insert(Children::from_entities(children));
        }
        self
    }

    fn remove_children(&mut self, children: &[Entity]) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            remove_children(parent, children, world);
        });
        self
    }

    fn set_parent(&mut self, parent: Entity) -> &mut Self {
        let child = self.id();
        self.world_scope(|world| {
            world.entity_mut(parent).add_child(child);
        });
        self
    }

    fn remove_parent(&mut self) -> &mut Self {
        let child = self.id();
        if let Some(parent) = self.take::<Parent>().map(|p| p.get()) {
            self.world_scope(|world| {
                remove_from_children(world, parent, child);
                push_events(world, [HierarchyEvent::ChildRemoved { child, parent }]);
            });
        }
        self
    }

    fn clear_children(&mut self) -> &mut Self {
        let parent = self.id();
        self.world_scope(|world| {
            clear_children(parent, world);
        });
        self
    }

    fn replace_children(&mut self, children: &[Entity]) -> &mut Self {
        self.clear_children().push_children(children)
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildChildren, BuildWorldChildren, HierarchyEvent};
    use crate::{children::Children, parent::Parent};
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

    fn assert_parent(world: &World, child: Entity, parent: Option<Entity>) {
        assert_eq!(world.get::<Parent>(child).map(|p| p.get()), parent);
    }

    fn assert_children(world: &World, parent: Entity, children: Option<&[Entity]>) {
        assert_eq!(world.get::<Children>(parent).map(|c| &**c), children);
    }

    fn omit_events(world: &mut World, number: usize) {
        let mut events_resource = world.resource_mut::<Events<HierarchyEvent>>();
        let mut events: Vec<_> = events_resource.drain().collect();
        events_resource.extend(events.drain(number..));
    }

    fn assert_events(world: &mut World, expected_events: &[HierarchyEvent]) {
        let events: Vec<_> = world
            .resource_mut::<Events<HierarchyEvent>>()
            .drain()
            .collect();
        assert_eq!(events, expected_events);
    }

    #[test]
    fn add_child() {
        let world = &mut World::new();
        world.insert_resource(Events::<HierarchyEvent>::default());

        let [a, b, c, d] = std::array::from_fn(|_| world.spawn_empty().id());

        world.entity_mut(a).add_child(b);

        assert_parent(world, b, Some(a));
        assert_children(world, a, Some(&[b]));
        assert_events(
            world,
            &[ChildAdded {
                child: b,
                parent: a,
            }],
        );

        world.entity_mut(a).add_child(c);

        assert_children(world, a, Some(&[b, c]));
        assert_parent(world, c, Some(a));
        assert_events(
            world,
            &[ChildAdded {
                child: c,
                parent: a,
            }],
        );
        world.entity_mut(d).add_child(b).add_child(c);
        assert_children(world, a, None);
    }

    #[test]
    fn set_parent() {
        let world = &mut World::new();
        world.insert_resource(Events::<HierarchyEvent>::default());

        let [a, b, c] = std::array::from_fn(|_| world.spawn_empty().id());

        world.entity_mut(a).set_parent(b);

        assert_parent(world, a, Some(b));
        assert_children(world, b, Some(&[a]));
        assert_events(
            world,
            &[ChildAdded {
                child: a,
                parent: b,
            }],
        );

        world.entity_mut(a).set_parent(c);

        assert_parent(world, a, Some(c));
        assert_children(world, b, None);
        assert_children(world, c, Some(&[a]));
        assert_events(
            world,
            &[ChildMoved {
                child: a,
                previous_parent: b,
                new_parent: c,
            }],
        );
    }

    #[test]
    fn set_parent_of_orphan() {
        let world = &mut World::new();

        let [a, b, c] = std::array::from_fn(|_| world.spawn_empty().id());
        world.entity_mut(a).set_parent(b);
        assert_parent(world, a, Some(b));
        assert_children(world, b, Some(&[a]));

        world.entity_mut(b).despawn();
        world.entity_mut(a).set_parent(c);

        assert_parent(world, a, Some(c));
        assert_children(world, c, Some(&[a]));
    }

    #[test]
    fn remove_parent() {
        let world = &mut World::new();
        world.insert_resource(Events::<HierarchyEvent>::default());

        let [a, b, c] = std::array::from_fn(|_| world.spawn_empty().id());

        world.entity_mut(a).push_children(&[b, c]);
        world.entity_mut(b).remove_parent();

        assert_parent(world, b, None);
        assert_parent(world, c, Some(a));
        assert_children(world, a, Some(&[c]));
        omit_events(world, 2);
        assert_events(
            world,
            &[ChildRemoved {
                child: b,
                parent: a,
            }],
        );

        world.entity_mut(c).remove_parent();
        assert_parent(world, c, None);
        assert_children(world, a, None);
        assert_events(
            world,
            &[ChildRemoved {
                child: c,
                parent: a,
            }],
        );
    }

    #[derive(Component)]
    struct C(u32);

    #[test]
    fn build_children() {
        let mut world = World::default();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let parent = commands.spawn(C(1)).id();
        let mut children = Vec::new();
        commands.entity(parent).with_children(|parent| {
            children.extend([
                parent.spawn(C(2)).id(),
                parent.spawn(C(3)).id(),
                parent.spawn(C(4)).id(),
            ]);
        });

        queue.apply(&mut world);
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.as_slice(),
            children.as_slice(),
        );
        assert_eq!(*world.get::<Parent>(children[0]).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(children[1]).unwrap(), Parent(parent));

        assert_eq!(*world.get::<Parent>(children[0]).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(children[1]).unwrap(), Parent(parent));
    }

    #[test]
    fn push_and_insert_and_remove_children_commands() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3), C(4), C(5)])
            .collect::<Vec<Entity>>();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(entities[0]).push_children(&entities[1..3]);
        }
        queue.apply(&mut world);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];
        let child3 = entities[3];
        let child4 = entities[4];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent).insert_children(1, &entities[3..]);
        }
        queue.apply(&mut world);

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child3, child4, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child3).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child4).unwrap(), Parent(parent));

        let remove_children = [child1, child4];
        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent).remove_children(&remove_children);
        }
        queue.apply(&mut world);

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child3, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert!(world.get::<Parent>(child1).is_none());
        assert!(world.get::<Parent>(child4).is_none());
    }

    #[test]
    fn push_and_clear_children_commands() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3), C(4), C(5)])
            .collect::<Vec<Entity>>();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(entities[0]).push_children(&entities[1..3]);
        }
        queue.apply(&mut world);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent).clear_children();
        }
        queue.apply(&mut world);

        assert!(world.get::<Children>(parent).is_none());

        assert!(world.get::<Parent>(child1).is_none());
        assert!(world.get::<Parent>(child2).is_none());
    }

    #[test]
    fn push_and_replace_children_commands() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3), C(4), C(5)])
            .collect::<Vec<Entity>>();

        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(entities[0]).push_children(&entities[1..3]);
        }
        queue.apply(&mut world);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];
        let child4 = entities[4];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        let replace_children = [child1, child4];
        {
            let mut commands = Commands::new(&mut queue, &world);
            commands.entity(parent).replace_children(&replace_children);
        }
        queue.apply(&mut world);

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child4];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child4).unwrap(), Parent(parent));
        assert!(world.get::<Parent>(child2).is_none());
    }

    #[test]
    fn push_and_insert_and_remove_children_world() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3), C(4), C(5)])
            .collect::<Vec<Entity>>();

        world.entity_mut(entities[0]).push_children(&entities[1..3]);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];
        let child3 = entities[3];
        let child4 = entities[4];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        world.entity_mut(parent).insert_children(1, &entities[3..]);
        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child3, child4, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child3).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child4).unwrap(), Parent(parent));

        let remove_children = [child1, child4];
        world.entity_mut(parent).remove_children(&remove_children);
        let expected_children: SmallVec<[Entity; 8]> = smallvec![child3, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert!(world.get::<Parent>(child1).is_none());
        assert!(world.get::<Parent>(child4).is_none());
    }

    #[test]
    fn push_and_insert_and_clear_children_world() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3)])
            .collect::<Vec<Entity>>();

        world.entity_mut(entities[0]).push_children(&entities[1..3]);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        world.entity_mut(parent).clear_children();
        assert!(world.get::<Children>(parent).is_none());
        assert!(world.get::<Parent>(child1).is_none());
        assert!(world.get::<Parent>(child2).is_none());
    }

    #[test]
    fn push_and_replace_children_world() {
        let mut world = World::default();
        let entities = world
            .spawn_batch(vec![C(1), C(2), C(3), C(4), C(5)])
            .collect::<Vec<Entity>>();

        world.entity_mut(entities[0]).push_children(&entities[1..3]);

        let parent = entities[0];
        let child1 = entities[1];
        let child2 = entities[2];
        let child3 = entities[3];
        let child4 = entities[4];

        let expected_children: SmallVec<[Entity; 8]> = smallvec![child1, child2];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert_eq!(*world.get::<Parent>(child1).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));

        world.entity_mut(parent).replace_children(&entities[2..]);
        let expected_children: SmallVec<[Entity; 8]> = smallvec![child2, child3, child4];
        assert_eq!(
            world.get::<Children>(parent).unwrap().0.clone(),
            expected_children
        );
        assert!(world.get::<Parent>(child1).is_none());
        assert_eq!(*world.get::<Parent>(child2).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child3).unwrap(), Parent(parent));
        assert_eq!(*world.get::<Parent>(child4).unwrap(), Parent(parent));
    }
}
