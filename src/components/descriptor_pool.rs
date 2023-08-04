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
		// TODO: Rust VMA
		let descriptor_pool_max = 1024 * frame_count as u32;
		let size_uniforms = vk::DescriptorPoolSize::builder()
			.ty(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(descriptor_pool_max)
			.build();
		let size_textures = vk::DescriptorPoolSize::builder()
			.ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(descriptor_pool_max)
			.build();
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
			.pool_sizes(&[size_uniforms, size_textures])
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