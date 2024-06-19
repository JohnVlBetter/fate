use glam::Vec3;
use scene::{
    component::{Component, ComponentBase, MeshRenderer},
    scene_tree::SceneTree,
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
    let mesh_renderer = scene_tree.get_component(node_id, MeshRenderer::get_pred());
    if let Component::MeshRenderer(mesh_renderer) = mesh_renderer.unwrap() {
        println!(
            "mesh_renderer has mesh {} {}",
            mesh_renderer.id, mesh_renderer.mesh
        );
    } else {
        println!("mesh_renderer has no mesh");
    }
    scene_tree.print_tree();
    let node = scene_tree.get_node(0);
    match &node.components()[0] {
        Component::Transform(mut transform) => {
            transform.with_translation(Vec3::new(1.0, 2.0, 3.0));
            println!("node 0 transform {:?}", transform.local_matrix());
        }
        _ => panic!("Expected Transform component"),
    };
    //scene_tree.update();
    println!("*****************");
    scene_tree.print_tree();
}
