use scene::{
    component::{Component, ComponentBase, MeshRenderer},
    scene_tree::SceneTree,
};

fn main() {
    let mut scene_tree = SceneTree::new();
    let node_id = scene_tree.create_node("nodeA", None);
    let component = MeshRenderer {
        id: 999,
        node_id,
        mesh: "mesh".to_string(),
    };
    scene_tree.add_component(
        node_id,
        scene::component::Component::MeshRenderer(component),
    );
    let mesh_renderer = scene_tree.get_component(node_id, MeshRenderer::get_pred());
    if let Component::MeshRenderer(mesh_renderer) = mesh_renderer.unwrap() {
        println!("mesh_renderer has mesh {} {}", mesh_renderer.id, mesh_renderer.mesh);
    } else {
        println!("mesh_renderer has no mesh");
    }
}
