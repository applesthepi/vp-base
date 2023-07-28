use ash::vk;

use crate::{Vertex, Device, VertexBuffer, IndexBuffer, BufferGO, Instance};

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
	pub fn new<V: Default + Copy + Clone>(
		instance: &Instance,
		device: &Device,
		vertices: &[V],
		indices: &[u32],
		indirect: &[vk::DrawIndexedIndirectCommand],
	) -> Self {
		let mut vb = BufferGO::new::<V>(
			instance,
			device,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			vertices.len(),
		);
		vb.update(instance, device, vertices);
		let mut ib = BufferGO::new::<u32>(
			instance,
			device,
			vk::BufferUsageFlags::INDEX_BUFFER,
			indices.len(),
		);
		ib.update(instance, device, indices);
		let mut indirect_b = BufferGO::new::<vk::DrawIndexedIndirectCommand>(
			instance,
			device,
			vk::BufferUsageFlags::INDIRECT_BUFFER,
			indirect.len(),
		);
		indirect_b.update(instance, device, indirect);
		Self {
			index_count: indices.len(),
			indirect_count: indirect.len(),
			vb,
			ib,
			indirect: indirect_b,
		}
	}

	pub fn update_vertices<V: Default + Copy + Clone>(
		&mut self,
		instance: &Instance,
		device: &Device,
		vertices: &[V],
	) {
		self.vb.update(instance, device, vertices);
	}

	pub fn update_indices(
		&mut self,
		instance: &Instance,
		device: &Device,
		indices: &[u32],
	) {
		self.index_count = indices.len();
		self.ib.update(instance, device, indices);
	}

	pub fn update_indirect(
		&mut self,
		instance: &Instance,
		device: &Device,
		indirect: &[vk::DrawIndexedIndirectCommand],
	) {
		self.indirect_count = indirect.len();
		self.indirect.update(instance, device, indirect);
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
			&[self.vb.memory.as_ref().unwrap_unchecked().0],
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
			self.ib.memory.as_ref().unwrap_unchecked().0,
			0,
			vk::IndexType::UINT32,
		);
	}}
	fn index_count(&self) -> usize {
		self.index_count
	}
}