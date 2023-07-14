use ash::vk;

use crate::{Vertex, Device, VertexBuffer, IndexBuffer, InstanceBuffer};

use super::{update_vertex_buffer, generate_vertex_buffer, generate_indirect_buffer, update_indirect_buffer, update_index_buffer, generate_index_buffer, generate_instance_buffer, update_instance_buffer};

#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_Instanced {
	pub index_count: usize,
	pub instance_count: usize,
	pub vb_gpu: vk::Buffer,
	pub vb_memory_gpu: vk::DeviceMemory,
	pub ib_gpu: vk::Buffer,
	pub ib_memory_gpu: vk::DeviceMemory,
	pub instance_gpu: vk::Buffer,
	pub instance_memory_gpu: vk::DeviceMemory,
}

impl GO_Instanced {
	pub fn new<V: Vertex + Copy, VI: Copy>(
		device: &Device,
		vertices_data: &[V],
		indices_data: &[u32],
		instance_data: &[VI],
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
		let (instance, instance_memory) = generate_instance_buffer(
			device,
			instance_data,
		);
		device.device.bind_buffer_memory(
			instance,
			instance_memory,
			0,
		).unwrap();
		Self {
			index_count: indices_data.len(),
			instance_count: instance_data.len(),
			vb_gpu: vb,
			vb_memory_gpu: vb_memory,
			ib_gpu: ib,
			ib_memory_gpu: ib_memory,
			instance_gpu: instance,
			instance_memory_gpu: instance_memory,
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

	pub fn update_instance<VI: Copy>(
		&mut self,
		device: &Device,
		instance: &[VI],
	) {
		self.instance_count = instance.len();
		update_instance_buffer(
			device,
			instance,
			self.instance_gpu,
			self.instance_memory_gpu,
		);
	}
}

impl VertexBuffer for GO_Instanced {
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

impl IndexBuffer for GO_Instanced {
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

impl InstanceBuffer for GO_Instanced {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_vertex_buffers(
			command_buffer,
			1,
			&[self.instance_gpu],
			&[0],
		);
	}}
	fn instance_count(&self) -> usize {
		self.instance_count
	}
}