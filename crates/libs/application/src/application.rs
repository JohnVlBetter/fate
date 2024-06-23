use std::sync::{Arc, Mutex};

use scene::scene::Scene;

pub struct Application {
    pub(crate) scene: Scene,
}

impl Application {
    fn get_instance() -> Arc<Mutex<Application>> {
        static mut APPLICATION: Option<Arc<Mutex<Application>>> = None;

        unsafe {
            APPLICATION
                .get_or_insert_with(|| {
                    Arc::new(Mutex::new(Self {
                        ..Default::default()
                    }))
                })
                .clone()
        }
    }
    pub fn new() -> Application {
        Application::default()
    }
    pub fn update(&mut self) {
        self.scene.update();
    }
}

impl Default for Application {
    fn default() -> Self {
        Application::new()
    }
}
