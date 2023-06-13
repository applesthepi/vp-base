use std::mem::align_of;

use ash::{vk, util::Align};

use crate::{Vertex, Device};

pub trait VertexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
}

// Vertex buffer with gpu only memory cached on cpu for future reference.
pub struct VertexBufferCGO<'a, V: Vertex> {
	pub vertices_cpu: &'a [V],
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

/// Vertex buffer with gpu only memory (nothing cached).
pub struct VertexBufferGO {
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

impl<'a, V: Vertex + Copy> VertexBufferCGO<'a, V> {
	pub fn new(
		device: &Device,
		vertices: &'a [V],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_buffer(
			device,
			vertices,
		);
		device.device.bind_buffer_memory(
			buffer,
			buffer_memory,
			0,
		).unwrap();
		Self {
			vertices_cpu: vertices,
			buffer_gpu: buffer,
			buffer_memory_gpu: buffer_memory,
		}
	}}

	pub fn update(
		&self,
		device: &Device,
		vertices: &'a [V],
	) {
		update_buffer(
			device,
			vertices,
			self.buffer_gpu,
			self.buffer_memory_gpu,
		);
	}
}

impl<'a, V: Vertex + Copy> VertexBuffer for VertexBufferCGO<'a, V> {
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

impl VertexBufferGO {
	pub fn new<V: Vertex + Copy>(
		device: &Device,
		vertices: &[V],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_buffer(
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
		update_buffer(
			device,
			vertices,
			self.buffer_gpu,
			self.buffer_memory_gpu,
		);
	}
}

impl VertexBuffer for VertexBufferGO {
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

fn generate_buffer<V: Vertex + Copy>(
	device: &Device,
	vertices: &[V],
) -> (vk::Buffer, vk::DeviceMemory) { unsafe {
	let buffer_info = vk::BufferCreateInfo::builder()
		.size((vertices.len() * std::mem::size_of::<V>()) as u64)
		.usage(vk::BufferUsageFlags::VERTEX_BUFFER)
		.sharing_mode(vk::SharingMode::EXCLUSIVE)
		.build();
	let buffer = device.device.create_buffer(
		&buffer_info,
		None,
	).unwrap();
	let buffer_requirements = device.device.get_buffer_memory_requirements(
		buffer,
	);
	let buffer_memory_index = Device::find_memory_type_index(
		&buffer_requirements,
		&device.physical_device_memory_properties,
		vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
	);
	let allocate_info = vk::MemoryAllocateInfo::builder()
		.allocation_size(buffer_requirements.size)
		.memory_type_index(buffer_memory_index)
		.build();
	let buffer_memory = device.device.allocate_memory(
		&allocate_info,
		None,
	).unwrap();
	update_buffer(
		device,
		vertices,
		buffer,
		buffer_memory,
	);
	(buffer, buffer_memory)
}}

fn update_buffer<V: Vertex + Copy>(
	device: &Device,
	vertices: &[V],
	buffer: vk::Buffer,
	buffer_memory: vk::DeviceMemory,
) { unsafe {
	let buffer_requirements = device.device.get_buffer_memory_requirements(
		buffer,
	);
	let mapped_memory = device.device.map_memory(
		buffer_memory,
		0,
		buffer_requirements.size,
		vk::MemoryMapFlags::empty(),
	).unwrap();
	let mut buffer_align = Align::new(
		mapped_memory,
		align_of::<V>() as u64,
		buffer_requirements.size,
	);
	buffer_align.copy_from_slice(vertices);
	device.device.unmap_memory(
		buffer_memory,
	);
}}