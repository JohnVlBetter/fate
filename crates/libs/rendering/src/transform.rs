use cgmath::{Deg, Euler, Matrix4, SquareMatrix, Vector3, Zero};

pub struct Transform {
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
    local_to_world_matrix: Matrix4<f32>,
    world_to_local_matrix: Matrix4<f32>,
    dirty: bool,
}

impl Transform {
    pub fn new(position: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Self {
        let translation_mat = Matrix4::from_translation(Vector3::from(position));
        let rotation_mat = Matrix4::from(Euler::new(
            Deg(rotation.x),
            Deg(rotation.y),
            Deg(rotation.z),
        ));
        let scale_mat = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        let local_to_world_matrix = translation_mat * rotation_mat * scale_mat;
        let world_to_local_matrix = local_to_world_matrix
            .invert()
            .unwrap_or(Matrix4::identity());
        Self {
            position,
            rotation,
            scale,
            local_to_world_matrix,
            world_to_local_matrix,
            dirty: false,
        }
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn rotation(&self) -> Vector3<f32> {
        self.rotation
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn translate(&mut self, position: Vector3<f32>) {
        self.position += position;
        self.dirty = true;
    }

    pub fn rotate(&mut self, rotation: Vector3<f32>) {
        self.rotation += rotation;
        self.dirty = true;
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
        self.dirty = true;
    }

    pub fn set_rotation(&mut self, rotation: Vector3<f32>) {
        self.rotation = rotation;
        self.dirty = true;
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) {
        self.scale = scale;
        self.dirty = true;
    }

    pub fn local_to_world_matrix(&mut self) -> Matrix4<f32> {
        if self.dirty {
            self.update();
        }
        self.local_to_world_matrix
    }

    pub fn world_to_local_matrix(&mut self) -> Matrix4<f32> {
        if self.dirty {
            self.update();
        }
        self.world_to_local_matrix
    }
}

impl Transform {
    fn update(&mut self) {
        let translation_mat = Matrix4::from_translation(Vector3::from(self.position));
        let rotation_mat = Matrix4::from(Euler::new(
            Deg(self.rotation.x),
            Deg(self.rotation.y),
            Deg(self.rotation.z),
        ));
        let scale_mat = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.local_to_world_matrix = translation_mat * rotation_mat * scale_mat;
        self.world_to_local_matrix = self
            .local_to_world_matrix
            .invert()
            .unwrap_or(Matrix4::identity());
        self.dirty = false;
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::zero(),
            rotation: Vector3::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            local_to_world_matrix: Matrix4::identity(),
            world_to_local_matrix: Matrix4::identity(),
            dirty: false,
        }
    }
}
