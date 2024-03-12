use vulkanalia::prelude::v1_0::*;

pub trait Vertex {
    fn binding_description() -> Vec<vk::VertexInputBindingDescription>;
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

impl Vertex for () {
    fn binding_description() -> Vec<vk::VertexInputBindingDescription> {
        vec![]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![]
    }
}
