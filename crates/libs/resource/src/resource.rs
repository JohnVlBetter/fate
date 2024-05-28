use std::any::Any;

pub trait Resource: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}