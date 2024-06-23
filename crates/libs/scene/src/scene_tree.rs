use glam::Affine3A;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{component::Component, transform::Transform};

pub struct Node {
    id: u32,
    name: String,
    pub components: RefCell<Vec<Rc<dyn Component>>>,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    pub fn new(name: String) -> Rc<Node> {
        let mut comps: Vec<Rc<dyn Component>> = vec![];
        comps.push(Rc::new(Transform::default()));
        Rc::new(Node {
            id: 0,
            name,
            components: RefCell::new(comps),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_child(parent: &Rc<Self>, child: &Rc<Self>) {
        child.parent.borrow_mut().upgrade().map(|old_parent| {
            old_parent
                .children
                .borrow_mut()
                .retain(|p| !Rc::ptr_eq(p, child));
        });

        *child.parent.borrow_mut() = Rc::downgrade(parent);

        parent.children.borrow_mut().push(Rc::clone(child));
    }

    pub fn remove_child(&self, index: usize) {
        self.children.borrow_mut().remove(index);
    }

    pub fn get_parent(&self) -> Option<Rc<Node>> {
        self.parent.borrow().upgrade()
    }

    pub fn get_child(&self, index: usize) -> Rc<Node> {
        Rc::clone(&self.children.borrow()[index])
    }

    pub fn children_count(&self) -> u32 {
        self.children.borrow().len() as u32
    }

    pub fn add_component(&self, component: Rc<dyn Component>) {
        self.components.borrow_mut().push(component);
    }

    pub fn remove_component<T: Component>(&self) {
        let mut remove_idx: i32 = -1;
        for (idx, component) in self.components.borrow().iter().enumerate() {
            if let Some(_) = component.as_any().downcast_ref::<T>() {
                remove_idx = idx as i32;
            }
        }
        if remove_idx >= 0 {
            self.components.borrow_mut().remove(remove_idx as usize);
        }
    }

    pub fn has_component<T: Component>(&self) -> bool {
        for component in self.components.borrow().iter() {
            if let Some(_) = component.as_any().downcast_ref::<T>() {
                return true;
            }
        }
        false
    }

    pub fn with_component<T: Component, F: FnOnce(&T)>(&self, f: F) {
        for component in self.components.borrow().iter() {
            if let Some(comp) = component.as_any().downcast_ref::<T>() {
                f(comp);
                return;
            }
        }
    }

    pub fn with_component_mut<T: Component, F: FnOnce(&mut T)>(&self, f: F) {
        for component in self.components.borrow_mut().iter_mut() {
            if let Some(comp) =
                Rc::get_mut(component).and_then(|c| c.as_any_mut().downcast_mut::<T>())
            {
                f(comp);
                return;
            }
        }
    }
}

pub struct SceneTree {
    root: Rc<Node>,
}

impl SceneTree {
    pub fn new() -> Self {
        let root = Node::new("Scene Root".to_string());
        SceneTree { root }
    }

    pub fn get_root_node(&self) -> Rc<Node> {
        Rc::clone(&self.root)
    }

    pub fn create_node(&self, name: String, parent: Option<Rc<Node>>) -> Rc<Node> {
        let node = Node::new(name);
        if let Some(parent) = parent {
            Node::add_child(&parent, &node);
        } else {
            Node::add_child(&self.root, &node);
        }
        node
    }

    pub fn update(&self) {
        let mut stack: Vec<(Affine3A, Rc<Node>)> = vec![];
        stack.push((Affine3A::IDENTITY, self.root.clone()));
        while let Some((parent_affine, node)) = stack.pop() {
            let mut cur_node_affine = Affine3A::IDENTITY;
            for (idx, component) in node.components.borrow_mut().iter_mut().enumerate() {
                let comp = Rc::get_mut(component);
                //更新transform
                if idx == 0 {
                    if let Some(transform) =
                        comp.and_then(|c| c.as_any_mut().downcast_mut::<Transform>())
                    {
                        transform.local_to_world_matrix = parent_affine * transform.local_matrix();
                        cur_node_affine = transform.local_to_world_matrix;
                    }
                }
                //更新其他组件
                else {
                    comp.unwrap().update();
                }
            }
            for child in node.children.borrow().iter() {
                stack.push((cur_node_affine, Rc::clone(child)));
            }
        }
    }
}

impl Default for SceneTree {
    fn default() -> Self {
        SceneTree::new()
    }
}
