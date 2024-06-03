use bevy_ecs::{component::Component, entity::Entity};
use core::slice;
use std::ops::Deref;
use smallvec::SmallVec;

#[derive(Component, Debug)]
pub struct Children(pub(crate) SmallVec<[Entity; 8]>);

impl Children {
    #[inline]
    pub(crate) fn from_entities(entities: &[Entity]) -> Self {
        Self(SmallVec::from_slice(entities))
    }
}

impl<'a> IntoIterator for &'a Children {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = slice::Iter<'a, Entity>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Deref for Children {
    type Target = [Entity];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}
