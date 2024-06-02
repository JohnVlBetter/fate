use bevy_ecs::{component::Component, entity::Entity};
use std::ops::Deref;

#[derive(Component, Debug, Eq, PartialEq)]
pub struct Parent(pub(crate) Entity);

impl Parent {
    #[inline(always)]
    pub fn get(&self) -> Entity {
        self.0
    }
}

impl Deref for Parent {
    type Target = Entity;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
