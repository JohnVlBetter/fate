use std::collections::HashMap;

use crate::component::Component;
use smallvec::SmallVec;

#[derive(Debug)]
pub struct Node {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) parent: Option<u32>,
    pub(crate) children: Vec<u32>,
    pub(crate) components: SmallVec<[u32; 8]>,
}

impl Node {
    pub(crate) fn new(id: u32, name: String) -> Self {
        Node {
            id,
            name,
            parent: None,
            children: Vec::new(),
            components: SmallVec::new(),
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

    pub fn spawn_node(&mut self, name: &str, parent_id: Option<u32>) -> u32 {
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
}
