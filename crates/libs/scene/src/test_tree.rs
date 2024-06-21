use std::any::Any;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node {
    id: u32,
    name: String,
    pub components: RefCell<Vec<Rc<dyn Component>>>,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

pub trait Component: Any {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
    fn start(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct Transform {
    pub id: u32,
}

impl Component for Transform {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        "Transform"
    }

    fn start(&mut self) {
        println!("Transform start");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Transform {
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }
}

pub struct MeshRenderer {
    pub id: u32,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        "MeshRenderer"
    }

    fn start(&mut self) {
        println!("MeshRenderer start");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Node {
    pub fn new() -> Rc<Node> {
        Rc::new(Node {
            id: 0,
            name: "Node".to_string(),
            components: RefCell::new(vec![]),
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

    pub fn get_child(&self, index: usize) -> Rc<Node> {
        Rc::clone(&self.children.borrow()[index])
    }

    pub fn add_component(&self, component: Rc<dyn Component>) {
        self.components.borrow_mut().push(component);
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
        for component in self.components.borrow().iter() {
            if let Some(comp) = component.as_any().downcast_ref::<RefCell<T>>() {
                f(&mut *comp.borrow_mut());
                return;
            }
        }
    }
}
