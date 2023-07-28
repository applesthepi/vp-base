use ash::vk;

use crate::{Vertex, Device, VertexBuffer, IndexBuffer, InstanceBuffer, BufferGO, Instance};

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
	pub fn new<V: Default + Copy + Clone, VI: Default + Copy + Clone>(
		instance: &Instance,
		device: &Device,
		vertices: &[V],
		indices: &[u32],
		instances: &[VI],
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
		let mut instance_b = BufferGO::new::<VI>(
			instance,
			device,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			instances.len(),
		);
		instance_b.update(instance, device, instances);
		Self {
			index_count: indices.len(),
			instance_count: instances.len(),
			vb,
			ib,
			instance: instance_b,
		}
	}

	pub fn with_capacity<V: Default + Copy + Clone, VI: Default + Copy + Clone>(
		instance: &Instance,
		device: &Device,
		vertex_count: usize,
		index_count: usize,
		instance_count: usize,
	) -> Self {
		let vb = BufferGO::new::<V>(
			instance,
			device,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			vertex_count,
		);
		let ib = BufferGO::new::<u32>(
			instance,
			device,
			vk::BufferUsageFlags::INDEX_BUFFER,
			index_count,
		);
		let instance_b = BufferGO::new::<VI>(
			instance,
			device,
			vk::BufferUsageFlags::VERTEX_BUFFER,
			instance_count,
		);
		Self {
			index_count: index_count,
			instance_count: instance_count,
			vb,
			ib,
			instance: instance_b,
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

	pub fn update_instances<VI: Default + Copy + Clone>(
		&mut self,
		instance: &Instance,
		device: &Device,
		instances: &[VI],
	) {
		self.instance_count = instances.len();
		self.instance.update(instance, device, instances);
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
			&[self.vb.memory.as_ref().unwrap_unchecked().0],
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
			self.ib.memory.as_ref().unwrap_unchecked().0,
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
			&[self.instance.memory.as_ref().unwrap_unchecked().0],
			&[0],
		);
	}}
	fn instance_count(&self) -> usize {
		self.instance_count
	}
}