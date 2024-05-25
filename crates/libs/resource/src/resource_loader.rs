use std::path::Path;

pub trait ResourceLoader: Send + Sync + 'static {
    fn load(&self, path: Path) -> Result<Box<dyn crate::resource::Resource>, Box<dyn std::error::Error>>;

    fn extensions(&self) -> &[&str];
}