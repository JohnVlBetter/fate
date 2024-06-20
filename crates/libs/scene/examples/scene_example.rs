/*use glam::Vec3;
use scene::{
    component::{Component, ComponentBase, MeshRenderer},
    scene_tree::SceneTree, transform::{self, Transform},
};

fn main() {
    let mut scene_tree = SceneTree::default();
    let node_id = scene_tree.create_node("nodeA", None);
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

use scene::test_tree::{MeshRenderer, Node, Transform};
use std::rc::Rc;

fn main() {
    let nodeA = Node::new();
    let nodeB = Node::new();
    Node::add_child(&nodeA, &nodeB);
    nodeA.add_component(Rc::new(Transform { id: 0 }));
    nodeA.add_component(Rc::new(MeshRenderer { id: 0 }));
    nodeA.get_component::<MeshRenderer>();
    if nodeA.has_component::<Transform>() {
        println!("nodeA has Transform component");
    }
    if nodeA.has_component::<MeshRenderer>() {
        println!("nodeA has MeshRenderer component");
    }
}
