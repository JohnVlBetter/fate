use std::collections::HashMap;

use crate::component::{Component, ComponentBase};
//use smallvec::SmallVec;

#[derive(Debug)]
pub struct Node {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) parent: Option<u32>,
    pub(crate) children: Vec<u32>,
    pub(crate) components: Vec<Component>,
}

impl Node {
    pub(crate) fn new(id: u32, name: String) -> Self {
        Node {
            id,
            name,
            parent: None,
            children: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent_id(&self) -> Option<u32> {
        self.parent
    }

    pub fn get_child(&self, index: usize) -> u32 {
        self.children[index]
    }

    pub fn children(&self) -> &[u32] {
        &self.children
    }

    pub fn components(&self) -> &[Component] {
        &self.components
    }
}

pub struct SceneTree {
    pub(crate) nodes: HashMap<u32, Node>,
    pub(crate) root: u32,
    id_allocator: u32,
}

impl SceneTree {
    pub fn new() -> Self {
        let mut nodes: HashMap<u32, Node> = HashMap::new();
        nodes.insert(0, Node::new(0, "Scene Root".to_string()));
        SceneTree {
            nodes,
            root: 0,
            id_allocator: 1,
        }
    }

    pub fn get_root_node() -> u32 {
        0
    }

    pub fn create_node(&mut self, name: &str, parent_id: Option<u32>) -> u32 {
        let id = self.id_allocator;
        self.id_allocator += 1;
        let mut node = Node::new(id, name.to_string());
        match parent_id {
            Some(parent_id) => {
                let parent = self
                    .nodes
                    .get_mut(&parent_id)
                    .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", parent_id));
                parent.children.push(id);
                node.parent = Some(parent_id);
            }
            None => {
                let root = self.nodes.get_mut(&0).unwrap();
                root.children.push(id);
                node.parent = Some(self.root);
            }
        }
        let mut transform = crate::component::Transform {
            id: 0,
            node_id: id,
            matrix: "Matrix".to_string(),
        };
        transform.start();
        let transform = Component::Transform(transform);
        node.components.push(transform);
        self.nodes.insert(id, node);
        id
    }

    pub fn destory_node(&mut self, id: u32) {
        let mut node = self
            .nodes
            .remove(&id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", id));
        node.components.iter_mut().for_each(|comp| {
            if let Component::Transform(transform) = comp {
                transform.destroy();
            } else if let Component::Camera(camera) = comp {
                camera.destroy();
            } else if let Component::Light(light) = comp {
                light.destroy();
            } else if let Component::MeshRenderer(mesh_renderer) = comp {
                mesh_renderer.destroy();
            }
        });
        //移除父节点的子节点
        if let Some(parent_id) = node.parent {
            let parent = self
                .nodes
                .get_mut(&parent_id)
                .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", parent_id));
            parent.children.remove(
                parent
                    .children
                    .iter()
                    .position(|child_id| *child_id == id)
                    .expect("没找到要删除的子节点!"),
            );
        }
    }

    pub fn get_node(&self, id: u32) -> &Node {
        self.nodes
            .get(&id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", id))
    }

    pub fn update(&mut self) {
        let mut satck: Vec<u32> = Vec::new();
        satck.push(self.root);
        while !satck.is_empty() {
            let node_id = satck.pop().unwrap();
            let node = self
                .nodes
                .get_mut(&node_id)
                .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
            node.components.iter_mut().for_each(|comp| {
                if let Component::Transform(transform) = comp {
                    transform.update();
                } else if let Component::Camera(camera) = comp {
                    camera.update();
                } else if let Component::Light(light) = comp {
                    light.update();
                } else if let Component::MeshRenderer(mesh_renderer) = comp {
                    mesh_renderer.update();
                }
            });
            for child_id in node.children() {
                satck.push(*child_id);
            }
        }
    }

    pub fn has_component(&mut self, node_id: u32, component: Component) -> bool {
        let node = self
            .nodes
            .get_mut(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        node.components
            .iter()
            .find(|&comp| comp == &component)
            .is_some()
    }

    pub fn add_component(&mut self, node_id: u32, component: Component) {
        let node = self
            .nodes
            .get_mut(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        if node
            .components
            .iter()
            .find(|&comp| comp == &component)
            .is_none()
        {
            node.components.push(component);
        } else {
            panic!("节点 {} 已经存在组件 {:?}", node_id, component);
        }
    }

    pub fn get_component(
        &mut self,
        node_id: u32,
        pred: impl Fn(&&Component) -> bool,
    ) -> Option<&Component> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        node.components.iter().find(pred)
    }
}

impl Default for SceneTree {
    fn default() -> Self {
        let mut tree = Self::new();
        let main_camera_node = tree.create_node("MainCamera", None);
        let main_light_node = tree.create_node("MainLight", None);
        let main_camera = Component::Camera(crate::component::Camera {
            id: 0,
            node_id: main_camera_node,
            view: "View".to_string(),
        });
        tree.add_component(main_camera_node, main_camera);
        let main_light = Component::Light(crate::component::Light {
            id: 0,
            node_id: main_light_node,
            color: "Color".to_string(),
        });
        tree.add_component(main_light_node, main_light);
        tree
    }
}
