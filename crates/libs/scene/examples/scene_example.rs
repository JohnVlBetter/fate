/*use glam::Vec3;
use scene::{
    component::{Component, ComponentBase, MeshRenderer},
    scene_tree::SceneTree, transform::{self, Transform},
};

fn main() {
    let mut scene_tree = SceneTree::default();
    let node_id = scene_tree.create_node("node_a", None);
    let component = MeshRenderer {
        id: 999,
        node_id,
        mesh: "cube mesh".to_string(),
    };
    scene_tree.add_component(
        node_id,
        scene::component::Component::MeshRenderer(component),
    );
    scene_tree.print_tree();
    let mut node = scene_tree.get_node(0);

    match &node.components()[0] {
        Component::Transform(mut transform) => {
            transform = transform.with_translation(Vec3::new(1.0, 2.0, 3.0));
            println!("node 0 transform {:?}", transform.local_matrix());
        }
        _ => panic!("Expected Transform component"),
    };
    scene_tree.update();
    println!("*****************");
    scene_tree.print_tree();
}
*/

use glam::Vec3;
use scene::{
    test_tree::SceneTree,
    test_tree::{MeshRenderer, Transform},
};
use std::rc::Rc;

fn main() {
    let scene_tree = SceneTree::default();
    let node_a = scene_tree.create_node("Node A".to_string(), None);
    let node_b = scene_tree.create_node("Node B".to_string(), None);
    let node_c = scene_tree.create_node("Node C".to_string(), Some(node_b.clone()));
    node_a.add_component(Rc::new(MeshRenderer { id: 0 }));
    node_a.remove_component::<MeshRenderer>();
    node_a.add_component(Rc::new(MeshRenderer { id: 0 }));
    if node_a.has_component::<Transform>() {
        println!("node_a has Transform component");
    }
    if node_a.has_component::<MeshRenderer>() {
        println!("node_a has MeshRenderer component");
    }
    if node_c.has_component::<Transform>() {
        println!("node_c has Transform component");
    }
    if node_c.has_component::<MeshRenderer>() {
        println!("node_c has MeshRenderer component");
    }
    scene_tree
        .get_root_node()
        .with_component_mut::<Transform, _>(|transform| {
            transform.id = 123456;
            transform.set_translation(Vec3::new(1.0, 2.0, 3.0));
        });
    node_a.with_component_mut::<Transform, _>(|transform| {
        transform.id = 19999;
    });
    node_b.with_component_mut::<Transform, _>(|transform| {
        transform.id = 19999;
        transform.set_translation(Vec3::new(1.0, 2.0, 3.0));
    });
    node_c.with_component_mut::<Transform, _>(|transform| {
        transform.id = 19999;
    });
    scene_tree.update();
    scene_tree
        .get_root_node()
        .with_component_mut::<Transform, _>(|transform| {
            println!(
                "root_node transform id: {} --- affine: {}",
                transform.clone().id,
                transform.local_matrix()
            );
        });
    node_a.with_component::<Transform, _>(|transform| {
        println!(
            "node_a transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    node_b.with_component::<Transform, _>(|transform| {
        println!(
            "node_b transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    node_c.with_component::<Transform, _>(|transform| {
        println!(
            "node_c transform affine: {}",
            transform.local_to_world_matrix()
        );
    });
    println!("children count: {}", node_a.children_count());
    println!("children count: {}", node_b.children_count());
    println!("children count: {}", node_c.children_count());
}
