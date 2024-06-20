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
    fn start(&mut self);
}

pub struct Transform {
    pub id: u32,
}

impl Component for Transform {
    fn id(&self) -> u32 {
        self.id
    }

    fn start(&mut self) {
        println!("Transform start");
    }
}

pub struct MeshRenderer {
    pub id: u32,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn start(&mut self) {
        println!("MeshRenderer start");
    }
}

pub fn print_any<T: Any>(value: &T) {
    let value_any = value as &dyn Any;

    if let Some(string) = value_any.downcast_ref::<String>() {
        println!("String ({}): {}", string.len(), string);
    } else if let Some(Transform { id: 0 }) = value_any.downcast_ref::<Transform>() {
        println!("Transform")
    } else if let Some(MeshRenderer { id: 0 }) = value_any.downcast_ref::<MeshRenderer>() {
        println!("MeshRenderer")
    } else {
        println!("{:?}", 1)
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

    /*pub fn get_component<T: Component>(&self) -> Option<Rc<T>> {
        for component in self.components.borrow().iter() {
            if let Some(component) = component.as_ref().downcast_ref::<T>() {
                return Some(Rc::clone(component));
            }
        }
        None
    }*/
}
