use std::mem::align_of;

use crate::{Device, Vertex};

mod vertex_go_indexed;
use ash::{vk, util::Align};
pub use vertex_go_indexed::*;
mod go_indirect;
pub use go_indirect::*;
mod index_go_indexed;
pub use index_go_indexed::*;

pub trait VertexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
}

pub trait IndexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
	fn index_count(&self) -> usize;
}

fn generate_vertex_buffer<V: Vertex + Copy>(
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
	update_vertex_buffer(// TODO: this is not called for index buffers; fix these functions
		device,
		vertices,
		buffer,
		buffer_memory,
	);
	(buffer, buffer_memory)
}}

fn update_vertex_buffer<V: Vertex + Copy>(
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

fn generate_index_buffer(
	device: &Device,
	indices: &[u32],
) -> (vk::Buffer, vk::DeviceMemory) { unsafe {
	let buffer_info = vk::BufferCreateInfo::builder()
		.size((indices.len() * std::mem::size_of::<u32>()) as u64)
		.usage(vk::BufferUsageFlags::INDEX_BUFFER)
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
	let mapped_memory = device.device.map_memory(
		buffer_memory,
		0,
		buffer_requirements.size,
		vk::MemoryMapFlags::empty(),
	).unwrap();
	let mut buffer_align = Align::new(
		mapped_memory,
		align_of::<u32>() as u64,
		buffer_requirements.size,
	);
	buffer_align.copy_from_slice(indices);
	device.device.unmap_memory(
		buffer_memory,
	);
	(buffer, buffer_memory)
}}

fn update_index_buffer(
	device: &Device,
	indices: &[u32],
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
		align_of::<u32>() as u64,
		buffer_requirements.size,
	);
	buffer_align.copy_from_slice(indices);
	device.device.unmap_memory(
		buffer_memory,
	);
}}

fn generate_indirect_buffer(
	device: &Device,
	indirect: &[vk::DrawIndexedIndirectCommand],
) -> (vk::Buffer, vk::DeviceMemory) { unsafe {
	let buffer_info = vk::BufferCreateInfo::builder()
		.size((indirect.len() * std::mem::size_of::<vk::DrawIndexedIndirectCommand>()) as u64)
		.usage(vk::BufferUsageFlags::INDIRECT_BUFFER)
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
	update_indirect_buffer(
		device,
		indirect,
		buffer,
		buffer_memory,
	);
	(buffer, buffer_memory)
}}

fn update_indirect_buffer(
	device: &Device,
	indirect: &[vk::DrawIndexedIndirectCommand],
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
		align_of::<vk::DrawIndexedIndirectCommand>() as u64,
		buffer_requirements.size,
	);
	buffer_align.copy_from_slice(indirect);
	device.device.unmap_memory(
		buffer_memory,
	);
}}