use std::mem::size_of;

use ash::vk;
use bytemuck::{Pod, Zeroable};

use crate::{Vertex, Device, VertexBuffer, IndexBuffer, BufferGO, Instance, RequirementType, ProgramData, BufferType};

#[repr(C)]
#[derive(Zeroable, Pod, Default, Copy, Clone)]
pub struct CrateDrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_Indirect {
	pub index_count: usize,
	pub indirect_count: usize,
	pub vb: BufferGO,
	pub ib: BufferGO,
	pub indirect: BufferGO,
}

impl GO_Indirect {
	pub fn new<V: Default + Copy + Clone + Pod>(
		program_data: &ProgramData,
		vertices: &[V],
		indices: &[u32],
		indirect: &[CrateDrawIndexedIndirectCommand],
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
		let mut indirect_b = BufferGO::new::<CrateDrawIndexedIndirectCommand>(
			program_data,
			RequirementType::Buffer(
				size_of::<u32>() * indices.len(),
				vk::BufferUsageFlags::INDIRECT_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		indirect_b.update(program_data, indirect);
		Self {
			index_count: indices.len(),
			indirect_count: indirect.len(),
			vb,
			ib,
			indirect: indirect_b,
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

	pub fn update_indirect(
		&mut self,
		program_data: &ProgramData,
		indirect: &[CrateDrawIndexedIndirectCommand],
	) {
		self.indirect_count = indirect.len();
		self.indirect.update(program_data, indirect);
	}
}

impl VertexBuffer for GO_Indirect {
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

impl IndexBuffer for GO_Indirect {
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