use std::ops::Mul;

use glam::{Affine3A, Mat3, Mat4, Quat, Vec3};

use crate::component::{Component, ComponentBase};

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub(crate) id: u32,
    pub(crate) node_id: u32,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
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
            local_to_world_matrix: Affine3A::from_translation(translation),
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
            local_to_world_matrix: Affine3A::from_rotation_translation(rotation, Vec3::ZERO),
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
            local_to_world_matrix: Affine3A::from_scale(scale),
            dirty: false,
        }
    }

    #[inline]
    pub fn calculate_local_to_world_matrix(&mut self) {
        if self.dirty {
            self.local_to_world_matrix = Affine3A::from_scale_rotation_translation(
                self.scale,
                self.rotation,
                self.translation,
            );
        }
    }

    #[inline]
    pub fn local_to_world_matrix(&mut self) -> Mat4 {
        self.calculate_local_to_world_matrix();
        Mat4::from(self.local_to_world_matrix)
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
            local_to_world_matrix: Affine3A::from_scale_rotation_translation(
                scale,
                rotation,
                translation,
            ),
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
    pub fn affine(&mut self) -> Affine3A {
        self.calculate_local_to_world_matrix();
        self.local_to_world_matrix
    }

    #[inline]
    #[must_use]
    pub fn mul_transform(&self, transform: Transform) -> Self {
        let translation = self.transform_point(transform.translation);
        let rotation = self.rotation * transform.rotation;
        let scale = self.scale * transform.scale;
        Transform {
            id: 0,
            node_id: 0,
            translation,
            rotation,
            scale,
            local_to_world_matrix: Affine3A::from_scale_rotation_translation(
                scale,
                rotation,
                translation,
            ),
            dirty: false,
        }
    }

    #[inline]
    pub fn transform_point(&self, mut point: Vec3) -> Vec3 {
        point = self.scale * point;
        point = self.rotation * point;
        point += self.translation;
        point
    }
}

impl Mul<Transform> for Transform {
    type Output = Transform;

    fn mul(self, transform: Transform) -> Self::Output {
        self.mul_transform(transform)
    }
}

impl Mul<Vec3> for Transform {
    type Output = Vec3;

    fn mul(self, value: Vec3) -> Self::Output {
        self.transform_point(value)
    }
}

impl ComponentBase for Transform {
    fn id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    fn get_pred() -> impl Fn(&&Component) -> bool {
        move |comp| {
            if let Component::Transform(_) = comp {
                true
            } else {
                false
            }
        }
    }

    fn get_node_id(&self) -> u32 {
        self.node_id
    }

    fn start(&mut self) {
        println!("node {} Transform start", self.node_id);
    }

    fn update(&mut self) {
        println!("node {} Transform update", self.node_id);
    }

    fn destroy(&mut self) {
        println!("node {} Transform destroy", self.node_id);
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
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }
}
