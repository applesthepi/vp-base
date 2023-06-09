use ash::vk;

pub trait Vertex {
	fn stride() -> u32;
	fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

pub trait OInstance {
	fn stride() -> u32;
}