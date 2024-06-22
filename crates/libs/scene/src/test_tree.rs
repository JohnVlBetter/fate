use glam::{Affine3A, Mat3, Mat4, Quat, Vec3};
use std::{
    any::Any,
    cell::RefCell,
    ops::Mul,
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

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub id: u32,
    pub node_id: u32,
    pub(crate) translation: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) scale: Vec3,
    pub(crate) local_matrix: Affine3A,
    pub(crate) local_to_world_matrix: Affine3A,
    pub(crate) dirty: bool,
}

impl PartialOrd for Transform {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
    #[inline]
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[inline]
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    #[inline]
    pub fn from_translation(translation: Vec3) -> Self {
        Transform {
            id: 0,
            node_id: 0,
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            local_matrix: Affine3A::from_translation(translation),
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }

    #[inline]
    pub fn from_rotation(rotation: Quat) -> Self {
        Transform {
            id: 0,
            node_id: 0,
            translation: Vec3::ZERO,
            rotation: rotation,
            scale: Vec3::ONE,
            local_matrix: Affine3A::from_rotation_translation(rotation, Vec3::ZERO),
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }

    #[inline]
    pub fn from_scale(scale: Vec3) -> Self {
        Transform {
            id: 0,
            node_id: 0,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: scale,
            local_matrix: Affine3A::from_scale(scale),
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }

    #[inline]
    pub fn local_matrix(&mut self) -> Affine3A {
        if self.dirty {
            println!("dirty");
            self.local_matrix = Affine3A::from_scale_rotation_translation(
                self.scale,
                self.rotation,
                self.translation,
            );
            self.dirty = false;
        }
        self.local_matrix
    }

    #[inline]
    pub fn local_to_world_matrix(&self) -> Affine3A {
        self.local_to_world_matrix
    }

    #[inline]
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Transform {
            id: 0,
            node_id: 0,
            translation,
            rotation,
            scale,
            local_matrix: Affine3A::from_scale_rotation_translation(scale, rotation, translation),
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }

    #[inline]
    #[must_use]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        self.look_at(target, up);
        self
    }

    #[inline]
    #[must_use]
    pub fn looking_to(mut self, direction: Vec3, up: Vec3) -> Self {
        self.look_to(direction, up);
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_translation(mut self, translation: Vec3) -> Self {
        self.translation = translation;
        self.dirty = true;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self.dirty = true;
        self
    }

    #[inline]
    #[must_use]
    pub const fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self.dirty = true;
        self
    }

    #[inline]
    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;
        self.dirty = true;
    }

    #[inline]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.dirty = true;
    }

    #[inline]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.dirty = true;
    }

    #[inline]
    pub fn translation(&self) -> Vec3 {
        self.translation
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    #[inline]
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    #[inline]
    pub fn local_x(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    #[inline]
    pub fn left(&self) -> Vec3 {
        -self.local_x()
    }

    #[inline]
    pub fn right(&self) -> Vec3 {
        self.local_x()
    }

    #[inline]
    pub fn local_y(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    #[inline]
    pub fn up(&self) -> Vec3 {
        self.local_y()
    }

    #[inline]
    pub fn down(&self) -> Vec3 {
        -self.local_y()
    }

    #[inline]
    pub fn local_z(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    #[inline]
    pub fn forward(&self) -> Vec3 {
        -self.local_z()
    }

    #[inline]
    pub fn back(&self) -> Vec3 {
        self.local_z()
    }

    #[inline]
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
        self.dirty = true;
    }

    #[inline]
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    #[inline]
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_x(angle));
    }

    #[inline]
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_y(angle));
    }

    #[inline]
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_z(angle));
    }

    #[inline]
    pub fn rotate_local(&mut self, rotation: Quat) {
        self.rotation *= rotation;
        self.dirty = true;
    }

    #[inline]
    pub fn rotate_local_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotate_local(Quat::from_axis_angle(axis, angle));
    }

    #[inline]
    pub fn rotate_local_x(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_x(angle));
    }

    #[inline]
    pub fn rotate_local_y(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_y(angle));
    }

    #[inline]
    pub fn rotate_local_z(&mut self, angle: f32) {
        self.rotate_local(Quat::from_rotation_z(angle));
    }

    #[inline]
    pub fn translate_around(&mut self, point: Vec3, rotation: Quat) {
        self.translation = point + rotation * (self.translation - point);
        self.dirty = true;
    }

    #[inline]
    pub fn rotate_around(&mut self, point: Vec3, rotation: Quat) {
        self.translate_around(point, rotation);
        self.rotate(rotation);
    }

    #[inline]
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        self.look_to(target - self.translation, up);
    }

    #[inline]
    pub fn look_to(&mut self, direction: Vec3, up: Vec3) {
        let back = -direction.try_normalize().unwrap_or(Vec3::NEG_Z);
        let up = up.try_normalize().unwrap_or(Vec3::Y);
        let right = up
            .cross(back)
            .try_normalize()
            .unwrap_or_else(|| up.any_orthonormal_vector());
        let up = back.cross(right);
        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, back));
        self.dirty = true;
    }

    #[inline]
    pub fn transform_point(&self, mut point: Vec3) -> Vec3 {
        point = self.scale * point;
        point = self.rotation * point;
        point += self.translation;
        point
    }
}

impl Mul<Vec3> for Transform {
    type Output = Vec3;

    fn mul(self, value: Vec3) -> Self::Output {
        self.transform_point(value)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            id: 0,
            node_id: 0,
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            local_matrix: Affine3A::IDENTITY,
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
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
            if let Some(transform) = Rc::get_mut(node.components.borrow_mut().get_mut(0).unwrap())
                .and_then(|c| c.as_any_mut().downcast_mut::<Transform>())
            {
                transform.local_to_world_matrix = parent_affine * transform.local_matrix();
                cur_node_affine = transform.local_to_world_matrix;
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
