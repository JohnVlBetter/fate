use glam::Vec3;

use crate::component::Component;
use std::any::Any;

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            min: Vec3::new(-0.5, -0.5, -0.5),
            max: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        if min.x > max.x || min.y > max.y || min.z > max.z {
            panic!("Invalid bounding box");
        }
        BoundingBox { min, max }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    pub fn is_empty(&self) -> bool {
        self.min == self.max
    }

    pub fn min(&self) -> Vec3 {
        self.min
    }

    pub fn max(&self) -> Vec3 {
        self.max
    }

    pub fn set_min(&mut self, min: Vec3) {
        self.min = min;
        self.check();
    }

    pub fn set_max(&mut self, max: Vec3) {
        self.max = max;
        self.check();
    }

    pub fn encapsulate_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    pub fn encapsulate_bounding_box(&mut self, other: BoundingBox) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    //https://iquilezles.org/articles/frustumcorrect/
    //cgmath_culling
    pub fn check_intersect_with_camera_frustum(&self) -> bool {
        // check box outside/inside of frustum
        return true;
    }

    fn check(&self) {
        if self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z {
            panic!("Invalid bounding box");
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MeshRenderer {
    id: u32,
    node_id: u32,
    mesh_id: u32,
    material_id: u32,
    visible: bool,
    cast_shadow: bool,
    receive_shadow: bool,
    bounding_box: BoundingBox,
}

impl Component for MeshRenderer {
    fn id(&self) -> u32 {
        self.id
    }

    fn name(&self) -> &str {
        "MeshRenderer"
    }

    fn start(&mut self) {}

    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl MeshRenderer {
    pub fn new(node_id: u32, mesh_id: u32, material_id: u32) -> Self {
        MeshRenderer {
            id: 0,
            node_id,
            mesh_id,
            material_id,
            visible: true,
            cast_shadow: true,
            receive_shadow: true,
            bounding_box: BoundingBox::default(),
        }
    }

    pub fn set_bounding_box(&mut self, bounding_box: BoundingBox) {
        self.bounding_box = bounding_box;
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.bounding_box
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn set_cast_shadow(&mut self, cast_shadow: bool) {
        self.cast_shadow = cast_shadow;
    }

    pub fn cast_shadow(&self) -> bool {
        self.cast_shadow
    }

    pub fn set_receive_shadow(&mut self, receive_shadow: bool) {
        self.receive_shadow = receive_shadow;
    }

    pub fn receive_shadow(&self) -> bool {
        self.receive_shadow
    }

    pub fn set_mesh_id(&mut self, mesh_id: u32) {
        self.mesh_id = mesh_id;
    }

    pub fn mesh_id(&self) -> u32 {
        self.mesh_id
    }

    pub fn set_material_id(&mut self, material_id: u32) {
        self.material_id = material_id;
    }

    pub fn material_id(&self) -> u32 {
        self.material_id
    }

    pub fn set_node_id(&mut self, node_id: u32) {
        self.node_id = node_id;
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn check_visibility_with_camera_frustum(&mut self) -> bool {
        //TODO 根据相机视锥体判断包围盒是否可见
        let res = self.bounding_box.check_intersect_with_camera_frustum();
        self.visible = res;
        res
    }
}

impl Default for MeshRenderer {
    fn default() -> Self {
        MeshRenderer {
            id: 0,
            node_id: 0,
            mesh_id: 0,
            material_id: 0,
            visible: true,
            cast_shadow: true,
            receive_shadow: true,
            bounding_box: BoundingBox::default(),
        }
    }
}
