use std::mem::align_of;

use ash::{vk, util::Align};

use crate::Device;

pub trait IndexBuffer {
	fn bind(&self, device: &Device, command_buffer: vk::CommandBuffer);
	fn index_count(&self) -> usize;
}

// Index buffer with gpu only memory cached on cpu for future reference.
pub struct IndexBufferCGO<'a> {
	pub indices_cpu: &'a [u32],
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

/// Index buffer with gpu only memory (nothing cached).
pub struct IndexBufferGO {
	pub index_count: usize,
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

impl<'a> IndexBufferCGO<'a> {
	pub fn new(
		device: &Device,
		indices: &'a [u32],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_buffer(
			device,
			indices,
		);
		device.device.bind_buffer_memory(
			buffer,
			buffer_memory,
			0,
		).unwrap();
		Self {
			indices_cpu: indices,
			buffer_gpu: buffer,
			buffer_memory_gpu: buffer_memory,
		}
	}}
}

impl<'a> IndexBuffer for IndexBufferCGO<'a> {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_index_buffer(
			command_buffer,
			self.buffer_gpu,
			0,
			vk::IndexType::UINT32,
		);
	}}
	fn index_count(&self) -> usize {
		self.indices_cpu.len()
	}
}

impl IndexBufferGO {
	pub fn new(
		device: &Device,
		indices: &[u32],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_buffer(
			device,
			indices,
		);
		device.device.bind_buffer_memory(
			buffer,
			buffer_memory,
			0,
		).unwrap();
		Self {
			index_count: indices.len(),
			buffer_gpu: buffer,
			buffer_memory_gpu: buffer_memory,
		}
	}}
}

impl IndexBuffer for IndexBufferGO {
	fn bind(
		&self,
		device: &Device,
		command_buffer: vk::CommandBuffer,
	) { unsafe {
		device.device.cmd_bind_index_buffer(
			command_buffer,
			self.buffer_gpu,
			0,
			vk::IndexType::UINT32,
		);
	}}
	fn index_count(&self) -> usize {
		self.index_count
	}
}

fn generate_buffer(
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