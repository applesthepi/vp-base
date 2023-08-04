
use std::mem::size_of;

use ash::vk;
use bytemuck::Pod;

use crate::{Vertex, Device, VertexBuffer, BufferGO, Instance, IndexBuffer, RequirementType, program_data, ProgramData, BufferType};

#[allow(non_camel_case_types)]
/// Vertex buffer with gpu only memory (nothing cached).
pub struct GO_Indexed {
	pub index_count: usize,
	pub vb: BufferGO,
	pub ib: BufferGO,
}

impl GO_Indexed {
	pub fn new<V: Default + Copy + Clone + Pod>(
		program_data: &ProgramData,
		vertices: &[V],
		indices: &[u32],
	) -> Self {
		let mut vb = BufferGO::new::<V>(
			program_data,
			RequirementType::Buffer(
				size_of::<V>() * vertices.len(),
				vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		vb.update(program_data, vertices);
		let mut ib = BufferGO::new::<u32>(
			program_data,
			RequirementType::Buffer(
				size_of::<u32>() * indices.len(),
				vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		ib.update(program_data, indices);
		Self {
			vb,
			ib,
			index_count: indices.len(),
		}
	}

	pub fn update_vertices<V: Default + Copy + Clone + Pod>(
		&mut self,
		program_data: &ProgramData,
		vertices: &[V],
	) {
		self.vb.update(program_data, vertices);
	}

	pub fn update_indices(
		&mut self,
		program_data: &ProgramData,
		indices: &[u32],
	) {
		self.index_count = indices.len();
		self.ib.update(program_data, indices);
	}
}

impl VertexBuffer for GO_Indexed {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		let buffer = match &self.vb.buffer {
			BufferType::Buffer(buffer) => buffer,
			BufferType::Image(_) => unreachable!(),
		};
		let offset = buffer.buffer_offset;
		let buffer = buffer.buffer;
		device.device.cmd_bind_vertex_buffers(
			command_buffer,
			0,
			&[buffer],
			// &[offset as u64],
			&[0],
		);
	}}
}

impl IndexBuffer for GO_Indexed {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		let buffer = match &self.ib.buffer {
			BufferType::Buffer(buffer) => buffer,
			BufferType::Image(_) => unreachable!(),
		};
		let offset = buffer.buffer_offset;
		let buffer = buffer.buffer;
		device.device.cmd_bind_index_buffer(
			command_buffer,
			buffer,
			// offset as u64,
			0,
			vk::IndexType::UINT32,
		);
	}}
	
	fn index_count(&self) -> usize {
		self.index_count
	}
}