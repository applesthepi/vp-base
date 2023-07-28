use std::mem::align_of;

use crate::{Device, Vertex};
use ash::{vk, util::Align};

mod go_indexed;
pub use go_indexed::*;
mod go_indirect;
pub use go_indirect::*;
mod go_instanced;
pub use go_instanced::*;

pub trait VertexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
}

pub trait IndexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
	fn index_count(&self) -> usize;
}

pub trait InstanceBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
	fn instance_count(&self) -> usize;
}