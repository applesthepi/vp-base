use ash::vk;

use crate::{CommandPool, Device};

pub struct CommandBuffer {
	pub command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
	pub fn new(
		device: &mut Device,
		command_pool: &mut CommandPool,
	) -> Self { unsafe {
		let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(command_pool.command_pool)
			.command_buffer_count(1)
			.level(vk::CommandBufferLevel::PRIMARY)
			.build();
		let command_buffer = device.device.allocate_command_buffers(
			&command_buffer_info
		).unwrap()[0];
		Self {
			command_buffer,
		}
	}}
}