use std::{ffi::c_void, mem::align_of, fmt::Display};
use std::ptr::copy_nonoverlapping as memcpy;

use ash::{vk, util::Align};
use bytemuck::{Pod, Zeroable, bytes_of};
use serde::Serialize;

use crate::{Device, Instance};

#[derive(Clone, Debug)]
pub struct FrameSet {
	pub buffer: vk::Buffer,
	pub memory: vk::DeviceMemory,
	pub set: vk::DescriptorSet,
	pub write: vk::WriteDescriptorSet,
	// pub mapped: *mut c_void,
}

#[derive(Clone)]
pub struct BlockState {
	pub layout: vk::DescriptorSetLayout,
	pub frame_sets: Vec<FrameSet>,
	pub binding: u32,
	pub set: u32,
	pub descriptor_size: usize,
}

impl BlockState {
	pub fn new(
		device: &Device,
		instance: &Instance,
		descriptor_pool: &vk::DescriptorPool,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		binding: u32,
		set: u32,
		descriptor_size: usize,
		descriptor_count: u32,
	) -> Self { unsafe {
		let mut frame_sets = BlockState::create_frame_sets(
			device,
			instance,
			frame_count,
			descriptor_size,
		);
		let (descriptor_sets, descriptor_writes) = BlockState::create_descriptor_sets(
			device,
			descriptor_pool,
			descriptor_set_layout,
			frame_count,
			&frame_sets,
			descriptor_size,
		);
		for (i, frame_set) in frame_sets.iter_mut().enumerate() {
			frame_set.set = descriptor_sets[i];
			frame_set.write = descriptor_writes[i];
		}
		Self {
			layout: *descriptor_set_layout,
			frame_sets,
			binding,
			set,
			descriptor_size,
		}
	}}

	pub fn update<T: Copy + Clone>(
		&self,
		device: &Device,
		data: &T,
		frame: Option<usize>,
	) { unsafe {
		// let data = bytes_of(data);
		if let Some(frame) = frame {
			let memory = device.device.map_memory(
				self.frame_sets[frame].memory,
				0,
				self.descriptor_size as u64,
				vk::MemoryMapFlags::empty(),
			).unwrap();
			memcpy(data, memory.cast(), 1);
			device.device.unmap_memory(
				self.frame_sets[frame].memory,
			);
		} else {
			panic!("non frame specific descriptor set updating is not finished; must specify a frame.");
		}
	}}

	pub fn destroy_memory(
		&mut self,
		device: &Device,
	) { unsafe {
		for frame_set in self.frame_sets.iter_mut() {
			device.device.free_memory(
				frame_set.memory,
				None,
			);
			device.device.destroy_buffer(
				frame_set.buffer,
				None,
			);
		}
	}}

	pub fn recreate_memory(
		&mut self,
		device: &Device,
		instance: &Instance,
		descriptor_pool: &vk::DescriptorPool,
		frame_count: usize,
	) { unsafe {
		self.frame_sets = BlockState::create_frame_sets(
			device,
			instance,
			frame_count,
			self.descriptor_size,
		);
		let (descriptor_sets, descriptor_writes) = BlockState::create_descriptor_sets(
			device,
			descriptor_pool,
			&self.layout,
			frame_count,
			&self.frame_sets,
			self.descriptor_size,
		);
		for (i, frame_set) in self.frame_sets.iter_mut().enumerate() {
			frame_set.set = descriptor_sets[i];
			frame_set.write = descriptor_writes[i];
		}
	}}
	
	fn create_frame_sets(
		device: &Device,
		instance: &Instance,
		frame_count: usize,
		descriptor_size: usize,
	) -> Vec<FrameSet> { unsafe {
		let mut frame_sets: Vec<FrameSet> = Vec::with_capacity(frame_count);
		for _ in 0..frame_count {
			let buffer_info = vk::BufferCreateInfo::builder()
				.size(descriptor_size as u64)
				.usage(
					vk::BufferUsageFlags::UNIFORM_BUFFER
				)
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				.build();
			let buffer = device.device.create_buffer(
				&buffer_info,
				None,
			).unwrap();
			let buffer_memory_requirement = device.device.get_buffer_memory_requirements(
				buffer,
			);
			let device_memory_properties = instance.instance.get_physical_device_memory_properties(
				device.physical_device
			);
			let buffer_memory_index = Device::find_memory_type_index(
				&buffer_memory_requirement,
				&device_memory_properties,
				vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
			);
			let allocate_info = vk::MemoryAllocateInfo::builder()
				.allocation_size(buffer_memory_requirement.size)
				.memory_type_index(buffer_memory_index)
				.build();
			let memory = device.device.allocate_memory(
				&allocate_info,
				None,
			).unwrap();
			device.device.bind_buffer_memory(
				buffer,
				memory,
				0,
			).unwrap();
			frame_sets.push(FrameSet {
				buffer,
				memory,
				set: vk::DescriptorSet::null(),
				write: vk::WriteDescriptorSet::default(),
			});
		}
		frame_sets
	}}

	fn create_descriptor_sets(
		device: &Device,
		descriptor_pool: &vk::DescriptorPool,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		frame_sets: &Vec<FrameSet>,
		descriptor_size: usize,
	) -> (Vec<vk::DescriptorSet>, Vec<vk::WriteDescriptorSet>) { unsafe {
		let layouts = vec![
			*descriptor_set_layout;
			frame_count
		];
		let info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&layouts)
			.build();
		let mut descriptor_sets = device.device.allocate_descriptor_sets(
			&info,
		).unwrap();
		let mut descriptor_writes = Vec::with_capacity(descriptor_sets.len());
		for (i, descriptor_set) in descriptor_sets.iter().enumerate() {
			let info = vk::DescriptorBufferInfo::builder()
				.buffer(frame_sets[i].buffer)
				.offset(0)
				.range(descriptor_size as u64)
				.build();
			let buffer_info = &[info];
			let write = vk::WriteDescriptorSet::builder()
				.dst_set(*descriptor_set)
				.dst_binding(0)
				.dst_array_element(0)
				.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
				.buffer_info(buffer_info)
				.build();
			device.device.update_descriptor_sets(
				&[write],
				&[] as &[vk::CopyDescriptorSet],
			);
			descriptor_writes.push(write);
		}
		(descriptor_sets, descriptor_writes)
	}}
}