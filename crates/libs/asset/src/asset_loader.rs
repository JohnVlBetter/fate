use std::sync::Arc;

pub trait AssetLoader: Send + Sync + 'static {
    fn load(&self, path: &str) -> Option<Arc<dyn crate::asset::Asset>>;

    fn extensions(&self) -> &[&str];
}
