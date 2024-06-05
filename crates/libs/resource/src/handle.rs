use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    path::Path,
};

use bevy_ecs::component::Component;
use bevy_utils::CowArc;

use crate::resource::Resource;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ResourceId<R: Resource> {
    index: u32,
    marker: PhantomData<fn() -> R>,
}

#[derive(Component)]
pub struct Handle<R: Resource> {
    pub(crate) resource_id: u32,
    pub(crate) resource_server_managed: bool,
    pub(crate) path: Option<CowArc<'static, Path>>,
    pub(crate) label: Option<CowArc<'static, str>>,
    marker: PhantomData<fn() -> R>,
}

impl<R: Resource> Drop for Handle<R> {
    fn drop(&mut self) {}
}

impl<R: Resource> std::fmt::Debug for Handle<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("resource handle")
            .field("resource id", &self.resource_id)
            .field("resource server managed", &self.resource_server_managed)
            .field("resource path", &self.path)
            .finish()
    }
}

impl<T: Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Handle {
            resource_id: self.resource_id,
            resource_server_managed: self.resource_server_managed,
            path: self.path.clone(),
            label: self.label.clone(),
            marker: PhantomData,
        }
    }
}

impl<R: Resource> Handle<R> {
    #[inline]
    pub fn id(&self) -> u32 {
        self.resource_id
    }

    #[inline]
    pub fn path(&self) -> Option<&CowArc<'static, Path>> {
        self.path.as_ref()
    }
}

impl<R: Resource> Default for Handle<R> {
    fn default() -> Self {
        Handle {
            resource_id: u32::MAX,
            resource_server_managed: false,
            path: None,
            label: None,
            marker: PhantomData,
        }
    }
}

impl<R: Resource> Hash for Handle<R> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id(), state);
    }
}

impl<R: Resource> PartialOrd for Handle<R> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id().cmp(&other.id()))
    }
}

impl<R: Resource> Ord for Handle<R> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id().cmp(&other.id())
    }
}

impl<R: Resource> PartialEq for Handle<R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<R: Resource> Eq for Handle<R> {}
