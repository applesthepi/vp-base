use std::{ffi::c_void, mem::align_of, fmt::Display};

use ash::{vk, util::Align};
use bytemuck::{Pod, Zeroable, bytes_of};
use serde::Serialize;

use crate::{Device, Instance};

#[derive(Clone, Debug)]
pub struct DescriptorSet {
	pub buffer: vk::Buffer,
	pub memory: vk::DeviceMemory,
	pub write: vk::WriteDescriptorSet,
	pub mapped: *mut c_void,
	pub set: vk::DescriptorSet,
}

#[derive(Clone)]
pub struct BlockState {
	pub layout: vk::DescriptorSetLayout,
	pub descriptor_buffers: Vec<DescriptorSet>,
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
		// let descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
		// 	.binding(binding)
		// 	.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
		// 	.stage_flags(vk::ShaderStageFlags::VERTEX)
		// 	.descriptor_count(descriptor_count)
		// 	.build();
		// let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
		// 	.bindings(&[
		// 		descriptor_set_layout_binding,
		// 	]).build();
		// let descriptor_set_layout = device.device.create_descriptor_set_layout(
		// 	&descriptor_set_layout_info,
		// 	None,
		// ).unwrap();
		let mut layouts = Vec::with_capacity(frame_count);
		layouts.resize(frame_count, *descriptor_set_layout);
		// for layout in layouts.iter_mut() {
		// 	let descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::builder()
		// 		.binding(binding)
		// 		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
		// 		.stage_flags(vk::ShaderStageFlags::VERTEX)
		// 		.descriptor_count(descriptor_count)
		// 		.build();
		// 	let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
		// 		.bindings(&[
		// 			descriptor_set_layout_binding,
		// 		]).build();
		// 	*layout = device.device.create_descriptor_set_layout(
		// 		&descriptor_set_layout_info,
		// 		None,
		// 	).unwrap();
		// }
		let mut descriptor_buffers: Vec<DescriptorSet> = Vec::with_capacity(frame_count);
		descriptor_buffers.resize(frame_count, DescriptorSet {
			buffer: vk::Buffer::null(),
			memory: vk::DeviceMemory::null(),
			mapped: std::ptr::null_mut(),
			write: vk::WriteDescriptorSet::default(),
			set: vk::DescriptorSet::null(),
		});
		for descriptor_buffer in descriptor_buffers.iter_mut() {
			let buffer_info = vk::BufferCreateInfo::builder()
				.size(descriptor_size as u64)
				.usage(
					vk::BufferUsageFlags::UNIFORM_BUFFER/* |
					vk::BufferUsageFlags::TRANSFER_DST*/
				)
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				// .queue_family_indices(&device.queue_family_index)
				.build();
			// buffer_info.queue_family_index_count = 1;
			// buffer_info.p_queue_family_indices = device.queue_family_index.as_ptr();
			// println!("{}", buffer_info.queue_family_index_count);
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
			let buffer_memory = device.device.allocate_memory(
				&allocate_info,
				None,
			).unwrap();
			let buffer_ptr = device.device.map_memory(
				buffer_memory,
				0,
				buffer_memory_requirement.size,
				vk::MemoryMapFlags::empty(),
			).unwrap();
			device.device.bind_buffer_memory(
				buffer,
				buffer_memory,
				0,
			).unwrap();
			descriptor_buffer.buffer = buffer;
			descriptor_buffer.memory = buffer_memory;
			descriptor_buffer.mapped = buffer_ptr;
		}
		let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(*descriptor_pool)
			.set_layouts(&layouts)
			.build();
		let descriptor_set_alloc = device.device.allocate_descriptor_sets(
			&descriptor_set_alloc_info,
		).unwrap();
		for (i, descriptor_set) in descriptor_buffers.iter_mut().enumerate() {
			println!("{:?}", descriptor_set.buffer);
			let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
				.buffer(descriptor_set.buffer)
				// .offset(0)
				.range(vk::WHOLE_SIZE)
				.build();
			let write_descriptor_set = vk::WriteDescriptorSet::builder()
				.dst_set(descriptor_set_alloc[i])
				.dst_binding(binding)
				// .dst_array_element(0)
				.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
				.buffer_info(&[descriptor_buffer_info])
				.build();
			device.device.update_descriptor_sets(
				&[write_descriptor_set],
				&[],
			);
			descriptor_set.write = write_descriptor_set;
			descriptor_set.set = descriptor_set_alloc[i];
		}
		println!("{:?}", descriptor_buffers);
		Self {
			layout: *descriptor_set_layout,
			descriptor_buffers,
			binding,
			set,
			descriptor_size,
		}
	}}

	pub fn update<T: Pod + Zeroable + Copy + Clone>(
		&self,
		device: &Device,
		command_buffer: &vk::CommandBuffer,
		data: &T,
		frame: Option<usize>,
	) { unsafe {
		let data = bytes_of(data);
		if let Some(frame) = frame {
			std::ptr::copy(data.as_ptr(), self.descriptor_buffers[frame].mapped.cast(), data.len());
			// device.device.cmd_update_buffer(*command_buffer, self.descriptor_buffers[frame].buffer, 0, data);
			device.device.update_descriptor_sets(
				&[self.descriptor_buffers[frame].write],
				&[],
			);
		} else {
			panic!("non frame specific descriptor set updating is not finished; must specify a frame.");
			// for descriptor_set in self.descriptor_buffers.iter() {
			// 	device.device.cmd_update_buffer(*command_buffer, descriptor_set.buffer, 0, data);
			// 	let descriptor_buffer_info = vk::DescriptorBufferInfo::builder()
			// 		.buffer(descriptor_set.buffer)
			// 		.offset(0)
			// 		.range(self.descriptor_size as u64)
			// 		.build();
			// 	let write_descriptor_set = vk::WriteDescriptorSet::builder()
			// 		.dst_set(descriptor_set.set)
			// 		.dst_binding(self.binding)
			// 		.dst_array_element(0)
			// 		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			// 		.buffer_info(&[descriptor_buffer_info])
			// 		.build();
			// 	device.device.update_descriptor_sets(
			// 		&[write_descriptor_set],
			// 		&[],
			// 	);
			// }
		}
	}}
}