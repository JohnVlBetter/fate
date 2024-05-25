use std::{collections::HashMap, sync::Arc};

use crate::resource_loader::ResourceLoader;
#[derive(Default)]
pub struct ResourceMgr {
    loaders: Vec<Arc<dyn ResourceLoader>>,
    extension_to_index: HashMap<String, usize>,
    type_name_to_index: HashMap<&'static str, usize>,
    preregistered_loaders: HashMap<&'static str, usize>,
}

impl ResourceMgr {
    pub fn new() -> Self {
        Self::new_with_loaders(Default::default())
    }

    pub(crate) fn new_with_loaders(loaders: Arc<dyn ResourceLoader>) -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn register_loader<L: ResourceLoader>(&self, loader: L) {
        let mut loaders = self.data.loaders.write();
        let type_name = std::any::type_name::<L>();
        let loader = Arc::new(loader);
        let (loader_index, is_new) =
            if let Some(index) = loaders.preregistered_loaders.remove(type_name) {
                (index, false)
            } else {
                (loaders.values.len(), true)
            };
        for extension in loader.extensions() {
            loaders
                .extension_to_index
                .insert(extension.to_string(), loader_index);
        }

        if is_new {
            loaders.type_name_to_index.insert(type_name, loader_index);
            loaders.values.push(MaybeAssetLoader::Ready(loader));
        } else {
            let maybe_loader = std::mem::replace(
                &mut loaders.values[loader_index],
                MaybeAssetLoader::Ready(loader.clone()),
            );
            match maybe_loader {
                MaybeAssetLoader::Ready(_) => unreachable!(),
                MaybeAssetLoader::Pending { sender, .. } => {
                    IoTaskPool::get()
                        .spawn(async move {
                            let _ = sender.broadcast(loader).await;
                        })
                        .detach();
                }
            }
        }
    }

    pub async fn get_asset_loader_with_extension(
        &self,
        extension: &str,
    ) -> Result<Arc<dyn ResourceLoader>, Box<dyn std::error::Error>> {
    }

    pub async fn get_asset_loader_with_type_name(
        &self,
        type_name: &str,
    ) -> Result<Arc<dyn ResourceLoader>, Box<dyn std::error::Error>> {
    }

    #[must_use = "not using the returned strong handle may result in the unexpected release of the asset"]
    pub fn load<'a, A: Resource>(&self, path: impl Into<Path<'a>>) -> Handle<A> {
        self.load_with_meta_transform(path, None)
    }

    pub fn preregister_loader<L: ResourceLoader>(&self, extensions: &[&str]) {
        let mut loaders = self.data.loaders.write();
        let loader_index = loaders.values.len();
        let type_name = std::any::type_name::<L>();
        loaders
            .preregistered_loaders
            .insert(type_name, loader_index);
        loaders.type_name_to_index.insert(type_name, loader_index);
        for extension in extensions {
            if loaders
                .extension_to_index
                .insert(extension.to_string(), loader_index)
                .is_some()
            {
            }
        }
    }
}
