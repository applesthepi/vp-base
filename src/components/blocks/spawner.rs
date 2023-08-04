use std::{sync::Arc, marker::PhantomData};

use ash::vk;

use crate::{Device, Instance, BlockState, BindingId, SetId, ProgramData};

pub trait BlockSpawnerGen {
	fn spawn(
		&self,
		program_data: &ProgramData,
		frame_count: usize,
	) -> Arc<BlockState>;
	
	fn layout(
		&self,
	) -> vk::DescriptorSetLayout;
}

pub struct BlockSpawner<B: Block> {
	_phantom: PhantomData<B>,
	layout: vk::DescriptorSetLayout,
	binding: BindingId,
	set: SetId,
}

impl<B: Block> BlockSpawner<B> {
	pub fn new(
		device: &Arc<Device>,
		binding: BindingId,
		set: SetId,
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
		program_data: &ProgramData,
		frame_count: usize,
	) -> Arc<BlockState> {
		B::create_block_state(
			program_data,
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

pub struct BlockSpawnerExist<B: Block> {
	_phantom: PhantomData<B>,
	layout: vk::DescriptorSetLayout,
	binding: BindingId,
	set: SetId,
}

impl<B: Block> BlockSpawnerExist<B> {
	pub fn new(
		spawner: Box<BlockSpawner<B>>,
	) -> Self {
		Self {
			_phantom: PhantomData,
			layout: spawner.layout,
			binding: spawner.binding,
			set: spawner.set,
		}
	}
}

impl<B: Block> BlockSpawnerGen for BlockSpawnerExist<B> {
	fn spawn(
		&self,
		program_data: &ProgramData,
		frame_count: usize,
	) -> Arc<BlockState> {
		B::create_block_state(
			program_data,
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
		program_data: &ProgramData,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		binding: BindingId,
		set: SetId,
	) -> Arc<BlockState>;

	fn create_descriptor_set_layout(
		device: &Arc<Device>,
		binding: BindingId,
	) -> vk::DescriptorSetLayout;
}