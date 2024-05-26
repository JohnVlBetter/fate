use std::sync::Arc;

pub trait ResourceLoader: Send + Sync + 'static {
    fn load(&self, path: &str) -> Option<Arc<dyn crate::resource::Resource>>;

    fn extensions(&self) -> &[&str];
}
