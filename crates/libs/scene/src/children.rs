use bevy_ecs::{component::Component, entity::Entity};
use core::slice;
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

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}