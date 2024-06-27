use glam::{Affine3A, Mat3, Mat4, Quat, Vec3};
use std::ops::Mul;

#[derive(Clone)]
pub struct Transform {
    pub(crate) id: u32,
    pub(crate) translation: Vec3,
    pub(crate) rotation: Quat,
    pub(crate) scale: Vec3,
    pub(crate) local_matrix: Affine3A,
    pub(crate) local_to_world_matrix: Affine3A,
    pub(crate) dirty: bool,
}

impl Transform {
    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

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
    pub fn set_local_to_world_matrix(&mut self, matrix: Affine3A) {
        self.local_to_world_matrix = matrix;
    }

    #[inline]
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();

        Transform {
            id: 0,
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
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            local_matrix: Affine3A::IDENTITY,
            local_to_world_matrix: Affine3A::IDENTITY,
            dirty: false,
        }
    }
}
