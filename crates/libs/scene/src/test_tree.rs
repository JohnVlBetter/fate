use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub struct Node {
    id: u32,
    name: String,
    components: Vec<u32>,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    pub fn new() -> Rc<Node> {
        Rc::new(Node {
            id: 0,
            name: "Node".to_string(),
            components: vec![],
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
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
}
