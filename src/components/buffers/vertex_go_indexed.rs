use ash::vk;

use crate::{Vertex, Device, VertexBuffer};

use super::{update_vertex_buffer, generate_vertex_buffer};

#[allow(non_camel_case_types)]
/// Vertex buffer with gpu only memory (nothing cached).
pub struct VB_GO_Indexed {
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

impl VB_GO_Indexed {
	pub fn new<V: Vertex + Copy>(
		device: &Device,
		vertices: &[V],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_vertex_buffer(
			device,
			vertices,
		);
		device.device.bind_buffer_memory(
			buffer,
			buffer_memory,
			0,
		).unwrap();
		Self {
			buffer_gpu: buffer,
			buffer_memory_gpu: buffer_memory,
		}
	}}

	pub fn update<V: Vertex + Copy>(
		&self,
		device: &Device,
		vertices: &[V],
	) {
		update_vertex_buffer(
			device,
			vertices,
			self.buffer_gpu,
			self.buffer_memory_gpu,
		);
	}
}

impl VertexBuffer for VB_GO_Indexed {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_vertex_buffers(
			command_buffer,
			0,
			&[self.buffer_gpu],
			&[0],
		);
	}}
}