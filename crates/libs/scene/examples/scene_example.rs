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

    let node = scene_tree.get_node(0);
    println!("node children: {}", node.children().len());
    println!("node components: {}", node.components().len());

    for child in node.children() {
        let child = scene_tree.get_node(*child);
        println!("child: {}", child.name());
        let components = child.components();
        for component in components {
            if let Component::MeshRenderer(mesh_renderer) = component {
                println!(
                    "mesh_renderer has mesh {} {}",
                    mesh_renderer.id, mesh_renderer.mesh
                );
            } else if let Component::Transform(transform) = component {
                println!("transform has matrix {} {}", transform.id, transform.matrix);
            } else if let Component::Camera(camera) = component {
                println!("camera has view {}", camera.view);
            } else if let Component::Light(light) = component {
                println!("light has color {}", light.color);
            } else {
                println!("unknown component");
            }
        }
    }
}
