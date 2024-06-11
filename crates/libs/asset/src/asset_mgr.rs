use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::{asset::Asset, asset_loader::AssetLoader};
#[derive(Default)]
pub struct AssetMgr {
    loaders: Vec<Arc<dyn AssetLoader>>,
    extension_to_index: HashMap<String, usize>,
    type_name_to_index: HashMap<&'static str, usize>,
    preregistered_loaders: HashMap<&'static str, usize>,
}

impl AssetMgr {
    pub fn register_loader<L: AssetLoader>(loader: L) {
        let binding = AssetMgr::get_instance();
        let mut mgr = binding.lock().unwrap();
        mgr.internal_register_loader(loader);
    }

    pub fn get_asset_loader_with_extension(extension: &str) -> Option<Arc<dyn AssetLoader>> {
        let binding = AssetMgr::get_instance();
        let mgr = binding.lock().unwrap();
        mgr.internal_get_asset_loader_with_extension(extension)
    }

    pub fn get_asset_loader_with_type_name(type_name: &str) -> Option<Arc<dyn AssetLoader>> {
        let binding = AssetMgr::get_instance();
        let mgr = binding.lock().unwrap();
        mgr.internal_get_asset_loader_with_type_name(type_name)
    }

    pub fn load(path: &Path) -> Option<Arc<dyn Asset>> {
        let binding = AssetMgr::get_instance();
        let mgr = binding.lock().unwrap();
        mgr.internal_load(path)
    }

    pub fn preregister_loader<L: AssetLoader>(extensions: &[&str]) {
        let binding = AssetMgr::get_instance();
        let mut mgr = binding.lock().unwrap();
        mgr.internal_preregister_loader::<L>(extensions)
    }

    fn get_instance() -> Arc<Mutex<AssetMgr>> {
        static mut ASSET_MGR: Option<Arc<Mutex<AssetMgr>>> = None;

        unsafe {
            ASSET_MGR
                .get_or_insert_with(|| {
                    Arc::new(Mutex::new(Self {
                        ..Default::default()
                    }))
                })
                .clone()
        }
    }

    fn internal_register_loader<L: AssetLoader>(&mut self, loader: L) {
        let type_name = std::any::type_name::<L>();
        let loader = Arc::new(loader);
        let (loader_index, is_new) =
            if let Some(index) = self.preregistered_loaders.remove(type_name) {
                (index, false)
            } else {
                (self.loaders.len(), true)
            };
        for extension in loader.extensions() {
            self.extension_to_index
                .insert(extension.to_string(), loader_index);
        }

        if is_new {
            self.type_name_to_index.insert(type_name, loader_index);
            self.loaders.push(loader);
        } else {
            let _ = std::mem::replace(&mut self.loaders[loader_index], loader);
        }
    }

    fn internal_get_asset_loader_with_extension(
        &self,
        extension: &str,
    ) -> Option<Arc<dyn AssetLoader>> {
        let index = self.extension_to_index.get(extension).unwrap();

        self.loaders.get(*index).cloned()
    }

    fn internal_get_asset_loader_with_type_name(
        &self,
        type_name: &str,
    ) -> Option<Arc<dyn AssetLoader>> {
        let index = self.type_name_to_index.get(type_name).unwrap();

        self.loaders.get(*index).cloned()
    }

    fn internal_load(&self, path: &Path) -> Option<Arc<dyn Asset>> {
        let extension = path.extension().unwrap().to_str().unwrap();
        let loader = self.internal_get_asset_loader_with_extension(extension)?;
        loader.load(path.to_str().unwrap())
    }

    fn internal_preregister_loader<L: AssetLoader>(&mut self, extensions: &[&str]) {
        let loader_index = self.loaders.len();
        let type_name = std::any::type_name::<L>();
        self.preregistered_loaders.insert(type_name, loader_index);
        self.type_name_to_index.insert(type_name, loader_index);
        for extension in extensions {
            if self
                .extension_to_index
                .insert(extension.to_string(), loader_index)
                .is_some()
            {}
        }
    }
}
