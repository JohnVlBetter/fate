use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::component::Component;
//use smallvec::SmallVec;

#[derive(Debug)]
pub struct Node {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) parent: Option<u32>,
    pub(crate) children: Vec<u32>,
    pub(crate) components: Vec<u32>, //SmallVec<[u32; 8]>,
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

    pub fn components(&self) -> &[u32] {
        &self.components
    }
}

pub struct SceneTree {
    pub(crate) nodes: HashMap<u32, Node>,
    pub(crate) components: HashMap<u32, Box<dyn Component>>,
    pub(crate) root: u32,
    id_allocator: u32,
}

impl SceneTree {
    pub fn new() -> Self {
        let mut nodes: HashMap<u32, Node> = HashMap::new();
        nodes.insert(0, Node::new(0, "Scene Root".to_string()));
        SceneTree {
            nodes,
            components: HashMap::new(),
            root: 0,
            id_allocator: 0,
        }
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
                node.parent = Some(self.root);
            }
        }
        self.nodes.insert(id, node);
        id
    }

    pub fn destory_node(&mut self, id: u32) {
        let node = self
            .nodes
            .remove(&id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", id));
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
        //移除所有component
        node.components.iter().for_each(|component_id| {
            self.components.remove(component_id);
        });
    }

    pub fn get_node(&self, id: u32) -> &Node {
        self.nodes
            .get(&id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", id))
    }

    pub fn add_component<C: Component + 'static>(&mut self, node_id: u32, mut component: C) -> u32 {
        let id = self.id_allocator;
        self.id_allocator += 1;
        component.set_id(id);
        let component = Box::new(component);
        let node = self
            .nodes
            .get_mut(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        self.components.insert(id, component);
        node.components.push(id);
        id
    }

    /*pub fn get_component<C: Component>(&self, node_id: u32) -> Option<&C> {
        let node = self
            .nodes
            .get(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        let component_id = node
            .components
            .iter()
            .find(|component_id| self.components.get(component_id).unwrap().type_id() == C::ID);
        component_id.map(|component_id| {
            self.components
                .get(component_id)
                .unwrap()
                .downcast_ref::<C>()
                .unwrap()
        })
    }*/

    pub fn get_component<C: Component + 'static>(&self, node_id: u32) {
        let node = self
            .nodes
            .get(&node_id)
            .unwrap_or_else(|| panic!("没有找到id为 {} 的节点!", node_id));
        node.components.iter().find(|component_id| {
            Self::is_component::<C>(self.components.get(component_id).unwrap())
        });
        //println!("{:?}", component_id);
    }

    fn is_component<C: Component + 'static>(s: &dyn Any) -> bool {
        if s.is::<C>() {
            println!("It's a C!");
            true
        } else {
            println!("Not a C...");
            false
        }
    }
}
/*
use std::any::Any;

trait Component: Any {}

struct MeshRenderer {}

impl Component for MeshRenderer {}

struct Scene {
    components: Vec<Box<dyn Component>>,
}

impl Scene {
    fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        for component in &self.components {
            if let Some(specified_type) = component.as_any().downcast_ref::<T>() {
                return Some(specified_type);
            }
        }
        None
    }
}

impl Component {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
*/
