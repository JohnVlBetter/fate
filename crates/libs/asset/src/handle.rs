use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    path::Path,
};

use bevy_ecs::component::Component;
use bevy_utils::CowArc;

use crate::asset::Asset;

#[derive(Component)]
pub struct Handle<A: Asset> {
    pub(crate) asset_id: u32,
    pub(crate) is_loaded: bool,
    pub(crate) path: Option<CowArc<'static, Path>>,
    pub(crate) label: Option<CowArc<'static, str>>,
    marker: PhantomData<fn() -> A>,
}

impl<A: Asset> Drop for Handle<A> {
    fn drop(&mut self) {}
}

impl<A: Asset> std::fmt::Debug for Handle<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("asset handle")
            .field("asset id", &self.asset_id)
            .field("is loaded", &self.is_loaded)
            .field("asset path", &self.path)
            .finish()
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            asset_id: self.asset_id,
            is_loaded: self.is_loaded,
            path: self.path.clone(),
            label: self.label.clone(),
            marker: PhantomData,
        }
    }
}

impl<A: Asset> Handle<A> {
    #[inline]
    pub fn id(&self) -> u32 {
        self.asset_id
    }

    #[inline]
    pub fn path(&self) -> Option<&CowArc<'static, Path>> {
        self.path.as_ref()
    }
}

impl<A: Asset> Default for Handle<A> {
    fn default() -> Self {
        Handle {
            asset_id: u32::MAX,
            is_loaded: false,
            path: None,
            label: None,
            marker: PhantomData,
        }
    }
}

impl<A: Asset> Hash for Handle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id(), state);
    }
}

impl<A: Asset> PartialOrd for Handle<A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id().cmp(&other.id()))
    }
}

impl<A: Asset> Ord for Handle<A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

impl<A: Asset> PartialEq for Handle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<A: Asset> Eq for Handle<A> {}
