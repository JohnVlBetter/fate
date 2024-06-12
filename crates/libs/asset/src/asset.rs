use std::{
    any::Any,
    sync::{atomic::AtomicU32, Arc},
};

use bevy_ecs::system::Resource;

pub trait Asset: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub struct AssetIndexAllocator {
    next_index: AtomicU32,
}

impl Default for AssetIndexAllocator {
    fn default() -> Self {
        Self {
            next_index: Default::default(),
        }
    }
}

impl AssetIndexAllocator {
    pub fn reserve(&self) -> u32 {
        self.next_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Resource)]
pub struct AssetStorage<A: Asset> {
    storage: Vec<Option<A>>,
    len: u32,
    allocator: Arc<AssetIndexAllocator>,
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        Self {
            len: 0,
            storage: Default::default(),
            allocator: Default::default(),
        }
    }
}

impl<A: Asset> AssetStorage<A> {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, asset: A) -> bool {
        self.allocator.reserve();
        let index = self.flush() - 1;
        let value = &mut self.storage[index as usize];
        let exists = value.is_some();
        if !exists {
            self.len += 1;
        }
        *value = Some(asset);
        exists
    }

    //todo: 回收废弃index再利用
    pub fn remove_dropped(&mut self, index: u32) -> Option<A> {
        self.remove_internal(index, |dense_storage| {
            dense_storage.storage[index as usize] = None;
        })
    }

    pub fn remove_still_alive(&mut self, index: u32) -> Option<A> {
        self.remove_internal(index, |_| {})
    }

    fn remove_internal(&mut self, index: u32, removed_action: impl FnOnce(&mut Self)) -> Option<A> {
        self.flush();
        let value = &mut self.storage[index as usize];
        let res = value.take().map(|value| {
            self.len -= 1;
            value
        });
        removed_action(self);
        res
    }

    pub fn get(&self, index: u32) -> Option<&A> {
        let value = self.storage.get(index as usize)?;
        value.as_ref()
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut A> {
        let value = self.storage.get_mut(index as usize)?;
        value.as_mut()
    }

    pub fn flush(&mut self) -> u32 {
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed);
        self.storage.resize_with(new_len as usize, || None);
        new_len
    }

    pub fn get_index_allocator(&self) -> Arc<AssetIndexAllocator> {
        self.allocator.clone()
    }

    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.storage.iter().enumerate().filter_map(
            |(i, v)| {
                if v.is_some() {
                    Some(i as u32)
                } else {
                    None
                }
            },
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &A)> {
        self.storage
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(value) => Some((i as u32, value)),
            })
    }
}
