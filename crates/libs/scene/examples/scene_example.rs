use glam::{EulerRot, Quat, Vec3};
use scene::{mesh_renderer::MeshRenderer, scene_tree::SceneTree};
use std::rc::Rc;

fn main() {
    let scene_tree = SceneTree::default();
    let node_a = scene_tree.create_node("Node A".to_string(), None);
    let node_b = scene_tree.create_node("Node B".to_string(), None);
    let node_c = scene_tree.create_node("Node C".to_string(), Some(node_b.clone()));
    node_a.add_component(Rc::new(MeshRenderer::default()));
    node_a.remove_component::<MeshRenderer>();
    node_a.add_component(Rc::new(MeshRenderer::default()));
    if node_a.has_component::<MeshRenderer>() {
        println!("node_a has MeshRenderer component");
    }
    if node_c.has_component::<MeshRenderer>() {
        println!("node_c has MeshRenderer component");
    }
    scene_tree
        .get_root_node()
        .with_transform_mut::<_>(|transform| {
            transform.set_translation(Vec3::new(1.0, 2.0, 3.0));
            transform.set_scale(Vec3::new(0.5, 0.5, 0.5));
            transform.set_rotation(Quat::from_euler(EulerRot::XYZ, 50.0, 0.0, 0.0));
        });
    node_b.with_transform_mut::<_>(|transform| {
        transform.set_translation(Vec3::new(-1.0, -2.0, -3.0));
        transform.set_scale(Vec3::new(2.0, 2.0, 2.0));
        transform.set_rotation(Quat::from_euler(EulerRot::XYZ, -50.0, 0.0, 0.0));
    });
    println!("update!!!!!!!!!!!!!!!");
    scene_tree.update();
    scene_tree
        .get_root_node()
        .with_transform_mut::<_>(|transform| {
            println!("root_node transform affine: {}", transform.local_matrix());
        });
    node_a.with_transform_mut::<_>(|transform| {
        println!(
            "node_a transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    node_b.with_transform_mut::<_>(|transform| {
        println!(
            "node_b transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    node_c.with_transform_mut::<_>(|transform| {
        println!(
            "node_c transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    println!("children count: {}", node_a.children_count());
    println!("children count: {}", node_b.children_count());
    println!("children count: {}", node_c.children_count());
    node_b.with_transform_mut::<_>(|transform| {
        let (s, r, t) = transform
            .local_to_world_matrix()
            .to_scale_rotation_translation();
        println!(
            "node_b transform scale: {:?}, rotation: {:?}, translation: {:?}",
            s,
            r.to_euler(EulerRot::XYZ),
            t
        );
    });
}
