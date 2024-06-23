use std::any::Any;

pub trait Component: Any {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
    fn start(&mut self);
    fn update(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
