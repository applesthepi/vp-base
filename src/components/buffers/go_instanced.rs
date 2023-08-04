use std::mem::size_of;

use ash::vk;
use bytemuck::Pod;

use crate::{Vertex, Device, VertexBuffer, IndexBuffer, InstanceBuffer, BufferGO, Instance, RequirementType, ProgramData, BufferType};

#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_Instanced {
	pub index_count: usize,
	pub instance_count: usize,
	pub vb: BufferGO,
	pub ib: BufferGO,
	pub instance: BufferGO,
}

impl GO_Instanced {
	pub fn new<V: Default + Copy + Clone + Pod, VI: Default + Copy + Clone + Pod>(
		program_data: &ProgramData,
		vertices: &[V],
		indices: &[u32],
		instances: &[VI],
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
		let mut instance_b = BufferGO::new::<VI>(
			program_data,
			RequirementType::Buffer(
				size_of::<VI>() * instances.len(),
				vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		instance_b.update(program_data, instances);
		Self {
			index_count: indices.len(),
			instance_count: instances.len(),
			vb,
			ib,
			instance: instance_b,
		}
	}

	pub fn with_capacity<V: Default + Copy + Clone + Pod, VI: Default + Copy + Clone + Pod>(
		program_data: &ProgramData,
		vertex_capacity: usize,
		index_capacity: usize,
		instance_capacity: usize,
	) -> Self {
		let mut vb = BufferGO::new::<V>(
			program_data,
			RequirementType::Buffer(
				size_of::<V>() * vertex_capacity,
				vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		let mut ib = BufferGO::new::<u32>(
			program_data,
			RequirementType::Buffer(
				size_of::<u32>() * index_capacity,
				vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		let mut instance_b = BufferGO::new::<VI>(
			program_data,
			RequirementType::Buffer(
				size_of::<VI>() * instance_capacity,
				vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		Self {
			index_count: 0,
			instance_count: 0,
			vb,
			ib,
			instance: instance_b,
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

	pub fn update_instances<VI: Default + Copy + Clone + Pod>(
		&mut self,
		program_data: &ProgramData,
		instances: &[VI],
	) {
		self.instance_count = instances.len();
		self.instance.update(program_data, instances);
	}
}

impl VertexBuffer for GO_Instanced {
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

impl IndexBuffer for GO_Instanced {
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

impl InstanceBuffer for GO_Instanced {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		let buffer = match &self.instance.buffer {
			BufferType::Buffer(buffer) => buffer,
			BufferType::Image(_) => unreachable!(),
		};
		let offset = buffer.buffer_offset;
		let buffer = buffer.buffer;
		device.device.cmd_bind_vertex_buffers(
			command_buffer,
			1,
			&[buffer],
			// &[offset as u64],
			&[0],
		);
	}}
	fn instance_count(&self) -> usize {
		self.instance_count
	}
}