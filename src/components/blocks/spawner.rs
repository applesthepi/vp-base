use std::{sync::Arc, marker::PhantomData};

use ash::vk;

use crate::{Device, Instance, BlockState};

pub trait BlockSpawnerGen {
	fn spawn(
		&self,
		device: &Device,
		instance: &Instance,
		descriptor_pool: &vk::DescriptorPool,
		frame_count: usize,
	) -> Arc<BlockState>;
	
	fn layout(
		&self,
	) -> vk::DescriptorSetLayout;
}

pub struct BlockSpawner<B: Block> {
	_phantom: PhantomData<B>,
	layout: vk::DescriptorSetLayout,
	binding: u32,
	set: u32,
}

impl<B: Block> BlockSpawner<B> {
	pub fn new(
		device: &Arc<Device>,
		binding: u32,
		set: u32,
	) -> Self {
		Self {
			_phantom: PhantomData,
			layout: B::create_descriptor_set_layout(device, binding),
			binding,
			set,
		}
	}
}

impl<B: Block> BlockSpawnerGen for BlockSpawner<B> {
	fn spawn(
		&self,
		device: &Device,
		instance: &Instance,
		descriptor_pool: &vk::DescriptorPool,
		frame_count: usize,
	) -> Arc<BlockState> {
		B::create_block_state(
			device,
			instance,
			descriptor_pool,
			&self.layout,
			frame_count,
			self.binding,
			self.set,
		)
	}

	fn layout(
		&self,
	) -> vk::DescriptorSetLayout {
		self.layout
	}
}

pub trait Block {
	fn create_block_state(
		device: &Device,
		instance: &Instance,
		descriptor_pool: &vk::DescriptorPool,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		binding: u32,
		set: u32,
	) -> Arc<BlockState>;

	fn create_descriptor_set_layout(
		device: &Arc<Device>,
		binding: u32,
	) -> vk::DescriptorSetLayout;
}