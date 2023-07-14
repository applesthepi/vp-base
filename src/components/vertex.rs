use ash::vk;

pub trait Vertex {
	fn stride() -> u32;
	fn binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
	fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

pub trait OInstance {
	fn stride() -> u32;
}