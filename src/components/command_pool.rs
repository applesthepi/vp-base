use std::marker::PhantomData;

use ash::vk;

use crate::{Device, CommandBuffer};

pub struct CommandPool {
	pub command_pool: vk::CommandPool,
	pub command_buffers: Vec<CommandBuffer>,
}

impl CommandPool {
	pub fn new(
		device: &Device,
	) -> Self { unsafe {
		let command_pool_info = vk::CommandPoolCreateInfo::builder()
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
			.queue_family_index(device.queue_family_index[0])
			.build();
		let command_pool = device.device.create_command_pool(
			&command_pool_info,
			None,
		).unwrap();
		Self {
			command_pool,
			command_buffers: Vec::with_capacity(1024),
		}
	}}
}