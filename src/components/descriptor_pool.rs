use std::sync::Arc;

use ash::vk;

use crate::Device;

pub struct DescriptorPool {
    pub descriptor_pool: vk::DescriptorPool,
}

impl DescriptorPool {
	pub fn new(
		device: &Device,
		frame_count: usize,
	) -> Self { unsafe {
		let descriptor_pool_max = 64 * frame_count as u32;
		let descriptor_pool_size = vk::DescriptorPoolSize::builder()
			.descriptor_count(descriptor_pool_max)
			.build();
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
			.pool_sizes(&[descriptor_pool_size])
			.max_sets(descriptor_pool_max)
			.build();
		let descriptor_pool = device.device.create_descriptor_pool(
			&descriptor_pool_info,
			None,
		).unwrap();
		Self {
			descriptor_pool,
		}
	}}
}