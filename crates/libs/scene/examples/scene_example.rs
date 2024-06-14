use scene::{component::MeshRenderer, scene_tree::SceneTree};

fn main() {
    let mut scene_tree = SceneTree::new();
    let node_id = scene_tree.create_node("nodeA", None);
    let component = MeshRenderer {
        id: 0,
        node_id,
        mesh: "mesh".to_string(),
    };
    scene_tree.add_component(node_id, component);
    scene_tree.get_component::<MeshRenderer>(node_id);
}
