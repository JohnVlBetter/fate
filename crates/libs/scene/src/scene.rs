use crate::scene_tree::SceneTree;

pub struct Scene {
    pub scene_tree: SceneTree,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            scene_tree: SceneTree::default(),
        }
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene::default()
    }

    pub fn update(&mut self) {
        self.scene_tree.update();
    }
}
