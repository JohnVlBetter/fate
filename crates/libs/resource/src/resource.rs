use std::{
    any::Any,
    iter::Enumerate,
    sync::{atomic::AtomicU32, Arc},
};

pub trait Resource: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

pub(crate) struct ResourceIndexAllocator {
    next_index: AtomicU32,
}

impl Default for ResourceIndexAllocator {
    fn default() -> Self {
        Self {
            next_index: Default::default(),
        }
    }
}

impl ResourceIndexAllocator {
    pub fn reserve(&self) -> u32 {
        self.next_index
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

struct DenseResourceStorage<R: Resource> {
    storage: Vec<Option<R>>,
    len: u32,
    allocator: Arc<ResourceIndexAllocator>,
}

impl<R: Resource> Default for DenseResourceStorage<R> {
    fn default() -> Self {
        Self {
            len: 0,
            storage: Default::default(),
            allocator: Default::default(),
        }
    }
}

impl<R: Resource> DenseResourceStorage<R> {
    pub(crate) fn len(&self) -> usize {
        self.len as usize
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn insert(&mut self, index: u32, asset: R) -> bool {
        self.flush();
        let value = &mut self.storage[index as usize];
        let exists = value.is_some();
        if !exists {
            self.len += 1;
        }
        *value = Some(asset);
        exists
    }

    //todo: 回收废弃index再利用
    pub(crate) fn remove_dropped(&mut self, index: u32) -> Option<R> {
        self.remove_internal(index, |dense_storage| {
            dense_storage.storage[index as usize] = None;
        })
    }

    pub(crate) fn remove_still_alive(&mut self, index: u32) -> Option<R> {
        self.remove_internal(index, |_| {})
    }

    fn remove_internal(&mut self, index: u32, removed_action: impl FnOnce(&mut Self)) -> Option<R> {
        self.flush();
        let value = &mut self.storage[index as usize];
        let res = value.take().map(|value| {
            self.len -= 1;
            value
        });
        removed_action(self);
        res
    }

    pub(crate) fn get(&self, index: u32) -> Option<&R> {
        let value = self.storage.get(index as usize)?;
        value.as_ref()
    }

    pub(crate) fn get_mut(&mut self, index: u32) -> Option<&mut R> {
        let value = self.storage.get_mut(index as usize)?;
        value.as_mut()
    }

    pub(crate) fn flush(&mut self) {
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed);
        self.storage.resize_with(new_len as usize, || None);
    }

    pub(crate) fn get_index_allocator(&self) -> Arc<ResourceIndexAllocator> {
        self.allocator.clone()
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = u32> + '_ {
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

    pub fn iter(&self) -> impl Iterator<Item = (u32, &R)> {
        self.storage
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(value) => Some((i as u32, value)),
            })
    }
}
