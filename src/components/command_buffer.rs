use ash::vk;

use crate::{CommandPool, Device, Swapchain};

pub struct CommandBuffer {
	pub command_buffer: vk::CommandBuffer,
	pub fence_submit: vk::Fence,
	pub present_queue: vk::Queue,
}

impl CommandBuffer {
	pub fn new(
		device: &mut Device,
		command_pool: &mut CommandPool,
		swapchain: &Swapchain,
	) -> Self { unsafe {
		let command_buffer_info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(command_pool.command_pool)
			.command_buffer_count(1)
			.level(vk::CommandBufferLevel::PRIMARY)
			.build();
		let command_buffer = device.device.allocate_command_buffers(
			&command_buffer_info
		).unwrap()[0];
		let present_queue = swapchain.present_queue;
		let fence_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED)
			.build();
		let fence_submit = device.device.create_fence(
			&fence_info,
			None,
		).unwrap();
		Self {
			command_buffer,
			present_queue,
			fence_submit,
		}
	}}
}