use ash::vk;

use crate::{Vertex, Device, VertexBuffer, IndexBuffer};

use super::{update_vertex_buffer, generate_vertex_buffer, generate_indirect_buffer, update_indirect_buffer, update_index_buffer, generate_index_buffer};

#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_Indirect {
	pub index_count: usize,
	pub indirect_count: usize,
	pub vb_gpu: vk::Buffer,
	pub vb_memory_gpu: vk::DeviceMemory,
	pub ib_gpu: vk::Buffer,
	pub ib_memory_gpu: vk::DeviceMemory,
	pub indirect_gpu: vk::Buffer,
	pub indirect_memory_gpu: vk::DeviceMemory,
}

impl GO_Indirect {
	pub fn new<V: Vertex + Copy>(
		device: &Device,
		vertices_data: &[V],
		indices_data: &[u32],
		indirect_data: &[vk::DrawIndexedIndirectCommand],
	) -> Self { unsafe {
		let (vb, vb_memory) = generate_vertex_buffer(
			device,
			vertices_data,
		);
		device.device.bind_buffer_memory(
			vb,
			vb_memory,
			0,
		).unwrap();
		let (ib, ib_memory) = generate_index_buffer(
			device,
			indices_data,
		);
		device.device.bind_buffer_memory(
			ib,
			ib_memory,
			0,
		).unwrap();
		let (indirect, indirect_memory) = generate_indirect_buffer(
			device,
			indirect_data,
		);
		device.device.bind_buffer_memory(
			indirect,
			indirect_memory,
			0,
		).unwrap();
		Self {
			index_count: indices_data.len(),
			indirect_count: indirect_data.len(),
			vb_gpu: vb,
			vb_memory_gpu: vb_memory,
			ib_gpu: ib,
			ib_memory_gpu: ib_memory,
			indirect_gpu: indirect,
			indirect_memory_gpu: indirect_memory,
		}
	}}

	pub fn update_vertices<V: Vertex + Copy>(
		&self,
		device: &Device,
		vertices: &[V],
	) {
		update_vertex_buffer(
			device,
			vertices,
			self.vb_gpu,
			self.vb_memory_gpu,
		);
	}

	pub fn update_indices(
		&mut self,
		device: &Device,
		indices: &[u32],
	) {
		self.index_count = indices.len();
		update_index_buffer(
			device,
			indices,
			self.ib_gpu,
			self.ib_memory_gpu,
		);
	}

	pub fn update_indirect(
		&mut self,
		device: &Device,
		indirect: &[vk::DrawIndexedIndirectCommand],
	) {
		self.indirect_count = indirect.len();
		update_indirect_buffer(
			device,
			indirect,
			self.indirect_gpu,
			self.indirect_memory_gpu,
		);
	}
}

impl VertexBuffer for GO_Indirect {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_vertex_buffers(
			command_buffer,
			0,
			&[self.vb_gpu],
			&[0],
		);
	}}
}

impl IndexBuffer for GO_Indirect {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_index_buffer(
			command_buffer,
			self.ib_gpu,
			0,
			vk::IndexType::UINT32,
		);
	}}
	fn index_count(&self) -> usize {
		self.index_count
	}
}