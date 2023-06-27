use ash::vk;

use crate::{Device, IndexBuffer};

use super::{generate_index_buffer, update_index_buffer};

#[allow(non_camel_case_types)]
/// Index buffer with gpu only memory (nothing cached).
pub struct IB_GO_Indexed {
	pub index_count: usize,
	pub buffer_gpu: vk::Buffer,
	pub buffer_memory_gpu: vk::DeviceMemory,
}

impl IB_GO_Indexed {
	pub fn new(
		device: &Device,
		indices: &[u32],
	) -> Self { unsafe {
		let (buffer, buffer_memory) = generate_index_buffer(
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

	pub fn update(
		&mut self,
		device: &Device,
		indices: &[u32],
	) {
		self.index_count = indices.len();
		update_index_buffer(
			device,
			indices,
			self.buffer_gpu,
			self.buffer_memory_gpu,
		);
	}
}

impl IndexBuffer for IB_GO_Indexed {
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