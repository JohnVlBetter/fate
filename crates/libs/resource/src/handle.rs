use std::{
    any::TypeId,
    hash::{Hash, Hasher},
    path::Path,
    sync::Arc,
};

use bevy_ecs::component::Component;
use bevy_utils::get_short_name;

use crate::resource::Resource;

pub struct StrongHandle {
    pub(crate) id: i32,
    pub(crate) asset_server_managed: bool,
    pub(crate) path: Option<Path>,
}

impl Drop for StrongHandle {
    fn drop(&mut self) {}
}

impl std::fmt::Debug for StrongHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrongHandle")
            .field("id", &self.id)
            .field("asset_server_managed", &self.asset_server_managed)
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Component)]
pub enum Handle<R: Resource> {
    Strong(Arc<StrongHandle>),
    Weak(AssetId<R>),
}

impl<T: Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        match self {
            Handle::Strong(handle) => Handle::Strong(handle.clone()),
            Handle::Weak(id) => Handle::Weak(*id),
        }
    }
}

impl<R: Resource> Handle<R> {
    pub const fn weak_from_u128(value: u128) -> Self {
        Handle::Weak(AssetId::Uuid {
            uuid: Uuid::from_u128(value),
        })
    }

    #[inline]
    pub fn id(&self) -> AssetId<R> {
        match self {
            Handle::Strong(handle) => handle.id.typed_unchecked(),
            Handle::Weak(id) => *id,
        }
    }

    #[inline]
    pub fn path(&self) -> Option<&AssetPath<'static>> {
        match self {
            Handle::Strong(handle) => handle.path.as_ref(),
            Handle::Weak(_) => None,
        }
    }

    #[inline]
    pub fn is_weak(&self) -> bool {
        matches!(self, Handle::Weak(_))
    }

    #[inline]
    pub fn is_strong(&self) -> bool {
        matches!(self, Handle::Strong(_))
    }

    #[inline]
    pub fn clone_weak(&self) -> Self {
        match self {
            Handle::Strong(handle) => Handle::Weak(handle.id.typed_unchecked::<R>()),
            Handle::Weak(id) => Handle::Weak(*id),
        }
    }

    #[inline]
    pub fn untyped(self) -> UntypedHandle {
        match self {
            Handle::Strong(handle) => UntypedHandle::Strong(handle),
            Handle::Weak(id) => UntypedHandle::Weak(id.untyped()),
        }
    }
}

impl<R: Resource> Default for Handle<R> {
    fn default() -> Self {
        Handle::Weak(AssetId::default())
    }
}

impl<R: Resource> std::fmt::Debug for Handle<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = get_short_name(std::any::type_name::<R>());
        match self {
            Handle::Strong(handle) => {
                write!(
                    f,
                    "StrongHandle<{name}>{{ id: {:?}, path: {:?} }}",
                    handle.id.internal(),
                    handle.path
                )
            }
            Handle::Weak(id) => write!(f, "WeakHandle<{name}>({:?})", id.internal()),
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
