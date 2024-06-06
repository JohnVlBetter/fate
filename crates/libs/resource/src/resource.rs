use std::{
    any::Any,
    sync::{atomic::AtomicU32, Arc},
};

pub trait Resource: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ResourceIndex {
    pub(crate) generation: u32,
    pub(crate) index: u32,
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
    pub fn reserve(&self) -> ResourceIndex {
        ResourceIndex {
            index: self
                .next_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            generation: 0,
        }
    }
}

#[derive(Default)]
enum Entry<R: Resource> {
    #[default]
    None,
    Some {
        value: Option<R>,
        generation: u32,
    },
}

struct DenseResourceStorage<R: Resource> {
    storage: Vec<Entry<R>>,
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

    pub(crate) fn insert(&mut self, index: ResourceIndex, asset: R) -> bool {
        self.flush();
        let entry = &mut self.storage[index.index as usize];
        if let Entry::Some { value, generation } = entry {
            if *generation == index.generation {
                let exists = value.is_some();
                if !exists {
                    self.len += 1;
                }
                *value = Some(asset);
                exists
            } else {
                false
            }
        } else {
            false
        }
    }

    pub(crate) fn remove_dropped(&mut self, index: ResourceIndex) -> Option<R> {
        self.remove_internal(index, |dense_storage| {
            dense_storage.storage[index.index as usize] = Entry::None;
        })
    }

    pub(crate) fn remove_still_alive(&mut self, index: ResourceIndex) -> Option<R> {
        self.remove_internal(index, |_| {})
    }

    fn remove_internal(
        &mut self,
        index: ResourceIndex,
        removed_action: impl FnOnce(&mut Self),
    ) -> Option<R> {
        self.flush();
        let value = match &mut self.storage[index.index as usize] {
            Entry::None => return None,
            Entry::Some { value, generation } => {
                if *generation == index.generation {
                    value.take().map(|value| {
                        self.len -= 1;
                        value
                    })
                } else {
                    return None;
                }
            }
        };
        removed_action(self);
        value
    }

    pub(crate) fn get(&self, index: ResourceIndex) -> Option<&R> {
        let entry = self.storage.get(index.index as usize)?;
        match entry {
            Entry::None => None,
            Entry::Some { value, generation } => {
                if *generation == index.generation {
                    value.as_ref()
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn get_mut(&mut self, index: ResourceIndex) -> Option<&mut R> {
        let entry = self.storage.get_mut(index.index as usize)?;
        match entry {
            Entry::None => None,
            Entry::Some { value, generation } => {
                if *generation == index.generation {
                    value.as_mut()
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn flush(&mut self) {
        let new_len = self
            .allocator
            .next_index
            .load(std::sync::atomic::Ordering::Relaxed);
        self.storage.resize_with(new_len as usize, || Entry::Some {
            value: None,
            generation: 0,
        });
    }

    pub(crate) fn get_index_allocator(&self) -> Arc<ResourceIndexAllocator> {
        self.allocator.clone()
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = ResourceIndex> + '_ {
        self.storage
            .iter()
            .enumerate()
            .filter_map(|(i, v)| match v {
                Entry::None => None,
                Entry::Some { value, generation } => {
                    if value.is_some() {
                        Some(ResourceIndex::from(ResourceIndex {
                            index: i as u32,
                            generation: *generation,
                        }))
                    } else {
                        None
                    }
                }
            })
    }
}