use glam::Affine3A;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{
    camera::Camera, component::Component, frustum::Frustum, mesh_renderer::MeshRenderer,
    transform::Transform,
};

pub struct Node {
    id: u32,
    name: String,
    components: RefCell<Vec<Rc<dyn Component>>>,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
    transform: RefCell<Rc<Transform>>,
}

impl Node {
    pub fn new(name: String) -> Rc<Node> {
        Rc::new(Node {
            id: 0,
            name,
            components: RefCell::new(vec![]),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
            transform: RefCell::new(Rc::new(Transform::default())),
        })
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
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

    pub fn with_transform_mut<F: FnOnce(&mut Transform)>(&self, f: F) {
        let mut binding = self.transform.borrow_mut();
        let transform = Rc::get_mut(&mut binding).unwrap();
        f(transform);
    }
}

pub struct SceneTree {
    root: Rc<Node>,
    main_camera: Rc<Node>,
}

impl SceneTree {
    pub fn new() -> Self {
        let root = Node::new("Scene Root".to_string());
        let main_camera = Node::new("Main Camera".to_string());
        main_camera.add_component(Rc::new(Camera::default()));
        Node::add_child(&root, &main_camera);
        SceneTree { root, main_camera }
    }

    pub fn get_root_node(&self) -> Rc<Node> {
        Rc::clone(&self.root)
    }

    pub fn get_main_camera(&self) -> Rc<Node> {
        Rc::clone(&self.main_camera)
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
        let mut frustum = Frustum::default();
        self.main_camera.with_component_mut::<Camera, _>(|cam| {
            frustum = cam.get_frustum();
        });

        let mut stack: Vec<(Affine3A, Rc<Node>)> = vec![];
        stack.push((Affine3A::IDENTITY, self.root.clone()));
        while let Some((parent_affine, node)) = stack.pop() {
            let mut cur_node_affine = Affine3A::IDENTITY;

            //更新transform
            node.with_transform_mut(|transform| {
                transform.local_to_world_matrix = parent_affine * transform.local_matrix();
                cur_node_affine = transform.local_to_world_matrix();
            });

            for component in node.components.borrow_mut().iter_mut() {
                //mesh 视锥体裁剪
                if let Some(mesh_renderer) = Rc::get_mut(component)
                    .and_then(|c| c.as_any_mut().downcast_mut::<MeshRenderer>())
                {
                    let bounding_box = mesh_renderer.bounding_box();
                    let visible =
                        frustum.is_bounding_box_visible(bounding_box.min(), bounding_box.max());
                    mesh_renderer.set_visible(visible);
                    println!("Mesh Renderer: {} visible: {}", node.name(), visible);
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
