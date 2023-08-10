use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::Arc;
use std::{ffi::c_void, mem::align_of, fmt::Display};
use std::ptr::copy_nonoverlapping as memcpy;

use ash::{vk, util::Align};
use bytemuck::{Pod, Zeroable, bytes_of};
use serde::Serialize;

use crate::{Device, Instance, GO_Image, program_data, ProgramData, GO_Uniform, BufferType, GO_ImageArray, ImageArrayState};

mod spawner;
pub use spawner::*;

pub struct BlockDescriptorData {
	pub set_id: SetId,
	pub descriptor_sets: Vec<vk::DescriptorSet>,
	pub frames: Vec<FrameDescriptorSet>,
}

#[derive(Clone)]
pub enum WriteDSInfo {
	Uniform(Vec<vk::DescriptorBufferInfo>),
	Image(Vec<vk::DescriptorImageInfo>),
}

#[derive(Clone)]
pub struct WriteDS {
	pub write: vk::WriteDescriptorSet,
	pub info: WriteDSInfo,
}

pub struct FrameDescriptorSet {
	pub descriptor_writes: Vec<WriteDS>,
	pub descriptors: Vec<FrameDescriptor>,
}

#[derive(Clone, Copy, Debug)]
pub struct BindingId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct SetId(pub u32);

pub struct DescriptorUniform {
	pub binding_id: BindingId,
	pub buffer: GO_Uniform,
	pub size: usize,
}

pub struct DescriptorImage {
	pub binding_id: BindingId,
	pub image: GO_Image,
}

pub struct DescriptorImageArray {
	pub binding_id: BindingId,
	pub image_array: GO_ImageArray,
}

pub enum FrameDescriptor {
	Uniform(DescriptorUniform),
	Image(DescriptorImage),
	ImageArray(DescriptorImageArray),
}

pub struct DDTypeUniform {
	pub binding: BindingId,
	pub size: usize,
}

pub struct DDTypeImage {
	pub binding: BindingId,
	pub file_abs: String,
}

pub struct DDTypeImageArray {
	pub binding: BindingId,
	pub ias: ImageArrayState,
}

pub enum DDType {
	Uniform(DDTypeUniform),
	Image(DDTypeImage),
	ImageArray(DDTypeImageArray),
}

pub struct DescriptorDescription {
	pub dd_types: Vec<DDType>,
}

impl DescriptorDescription {
	pub fn new(
		types: Vec<DDType>,
	) -> Self {
		Self {
			dd_types: types,
		}
	}
}
   
// TODO: reorginize this for each type of descriptor with all the descriptor sets needed.
// #[derive(Clone, Debug)]
// pub struct FrameSet {
// 	pub buffer: vk::Buffer,
// 	pub memory: vk::DeviceMemory,
// 	pub set: vk::DescriptorSet,
// 	pub writes: Vec<vk::WriteDescriptorSet>,
// }

pub struct BlockState {
	pub layouts: Vec<vk::DescriptorSetLayout>,
	pub descriptor_data: BlockDescriptorData,
	pub descriptor_description: DescriptorDescription,
}

impl BlockState {
	pub fn new(
		program_data: &ProgramData,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		set_id: SetId,
		descriptor_description: DescriptorDescription,
	) -> Self { unsafe {

		// OTS CMD BUFFER
		
		let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(program_data.command_pool.command_pool)
			.command_buffer_count(1)
			.build();
		let cmd_buffer = *program_data.device.device.allocate_command_buffers(&cmd_alloc_info).unwrap().first().unwrap_unchecked();
		let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
			.build();
		program_data.device.device.begin_command_buffer(cmd_buffer, &cmd_begin_info).unwrap();

		// BUFFERS & WRITES

		let mut descriptor_data = BlockState::create_buffers(
			program_data,
			cmd_buffer,
			frame_count,
			&descriptor_description,
			set_id,
		);
		let layouts = BlockState::create_writes(
			program_data,
			descriptor_set_layout,
			frame_count,
			&mut descriptor_data,
		);

		// SUBMIT OTS CMD BUFFER

		program_data.device.device.end_command_buffer(cmd_buffer).unwrap();
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&[cmd_buffer])
			.build();
		program_data.device.device.queue_submit(program_data.swapchain.present_queue, &[submit_info], vk::Fence::null()).unwrap();
		Self {
			layouts,
			descriptor_data,
			descriptor_description,
		}
	}}

	pub fn update<T: Copy + Clone + Pod + Zeroable>(
		&self,
		device: &Device,
		data: &T,
		frame: Option<usize>,
	) { unsafe {
		// let data = bytes_of(data);
		if let Some(frame) = frame {
			let frame = &self.descriptor_data.frames[frame];
			for descriptor in frame.descriptors.iter() {
				match descriptor {
					FrameDescriptor::Uniform(uniform) => {
						match &uniform.buffer.buffer.buffer {
							BufferType::Buffer(buffer) => {
								let bytes = bytemuck::bytes_of(data);
								buffer.mapped.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
								// let mapped = buffer.buffer_allocation_info.get_mapped_data();
								// memcpy(data, mapped.cast(), 1);
							},
							BufferType::Image(_) => unreachable!(),
						}
					},
					FrameDescriptor::Image(_) => {},
					FrameDescriptor::ImageArray(_) => {},
				}
			}
		} else {
			unimplemented!("non frame specific descriptor set updating is not finished; must specify a frame.");
		}
	}}

	pub fn destroy_memory(
		&mut self,
		program_data: &ProgramData,
	) { unsafe {
		for frame_set in self.descriptor_data.frames.iter_mut() {
			for descriptor in frame_set.descriptors.iter() {
				match descriptor {
					FrameDescriptor::Uniform(uniform) => {
						let buffer = match &uniform.buffer.buffer.buffer {
							BufferType::Buffer(buffer) => buffer,
							BufferType::Image(_) => unreachable!(),
						};
						program_data.get_allocator().destroy_buffer(
							buffer.buffer,
							&buffer.buffer_allocation,
						);
					},
					FrameDescriptor::Image(_) => {},
					FrameDescriptor::ImageArray(_) => {},
				}
			}
		}
	}}

	pub fn recreate_memory(
		&mut self,
		program_data: &ProgramData,
		frame_count: usize,
	) { unsafe {
		
		// OTS CMD BUFFER

		let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(program_data.command_pool.command_pool)
			.command_buffer_count(1)
			.build();
		let cmd_buffer = *program_data.device.device.allocate_command_buffers(&cmd_alloc_info).unwrap().first().unwrap_unchecked();
		let cmd_begin_info = vk::CommandBufferBeginInfo::builder()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
			.build();
		program_data.device.device.begin_command_buffer(cmd_buffer, &cmd_begin_info).unwrap();

		// RECREATE BUFFERS
		
		let mut descriptor_data = BlockState::create_buffers(
			program_data,
			cmd_buffer,
			frame_count,
			&self.descriptor_description,
			self.descriptor_data.set_id,
		);
		let layouts = BlockState::create_writes(
			program_data,
			self.layouts.first().unwrap_unchecked(),
			frame_count,
			&mut descriptor_data,
		);
		self.descriptor_data = descriptor_data;
		self.layouts = layouts;

		// SUBMIT OTS CMD BUFFER

		program_data.device.device.end_command_buffer(cmd_buffer).unwrap();
		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(&[cmd_buffer])
			.build();
		program_data.device.device.queue_submit(program_data.swapchain.present_queue, &[submit_info], vk::Fence::null()).unwrap();
	}}
	
	fn create_buffers(
		program_data: &ProgramData,
		cmd_buffer: vk::CommandBuffer,
		frame_count: usize,
		descriptor_description: &DescriptorDescription,
		set_id: SetId,
	) -> BlockDescriptorData { unsafe {
		let mut frames: Vec<FrameDescriptorSet> = Vec::with_capacity(frame_count);
		for _ in 0..frame_count {
			let mut descriptors: Vec<FrameDescriptor> = Vec::with_capacity(8);
			for description in descriptor_description.dd_types.iter() {
				let frame = match description {
					DDType::Uniform(dd_type) => {
						create_uniform_buffer(
							program_data,
							dd_type,
						)
					},
					DDType::Image(dd_type) => {
						create_image_buffer(
							program_data,
							&cmd_buffer,
							dd_type,
						)
					},
					DDType::ImageArray(dd_type) => {
						create_image_array_buffer(
							program_data,
							&cmd_buffer,
							dd_type,
						)
					},
				};
				descriptors.push(frame);
			}
			frames.push(FrameDescriptorSet {
				descriptor_writes: Vec::with_capacity(descriptors.len()),
				descriptors,
			});
		}
		BlockDescriptorData {
			descriptor_sets: Vec::with_capacity(frames.len()),
			set_id,
			frames,
		}
	}}

	fn create_writes(
		program_data: &ProgramData,
		descriptor_set_layout: &vk::DescriptorSetLayout,
		frame_count: usize,
		descriptor_data: &mut BlockDescriptorData,
	) -> Vec<vk::DescriptorSetLayout> { unsafe {
		let layouts = vec![
			*descriptor_set_layout;
			frame_count
		];
		let info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(program_data.descriptor_pool.descriptor_pool)
			.set_layouts(&layouts)
			.build();
		descriptor_data.descriptor_sets = program_data.device.device.allocate_descriptor_sets(
			&info,
		).unwrap();
		for (i, frame) in descriptor_data.frames.iter_mut().enumerate() {
			for descriptor in frame.descriptors.iter_mut() {
				let write = match descriptor {
					FrameDescriptor::Uniform(uniform) => {
						create_write_uniform(
							uniform,
							descriptor_data.descriptor_sets[i],
						)
					},
					FrameDescriptor::Image(image) => {
						create_write_image(
							image,
							descriptor_data.descriptor_sets[i],
						)
					},
					FrameDescriptor::ImageArray(image_array) => {
						create_write_image_array(
							image_array,
							descriptor_data.descriptor_sets[i],
						)
					},
					_ => { unimplemented!(); }
				};
				frame.descriptor_writes.push(write);
			}
			// println!("{:?}", frame.descriptor_writes[0].);
			let writes: Vec<vk::WriteDescriptorSet> = frame.descriptor_writes.iter().map(
				|x| x.write
			).collect();
			program_data.device.device.update_descriptor_sets(
				&writes,
				&[] as &[vk::CopyDescriptorSet],
			);
		}
		layouts
	}}
}

fn create_uniform_buffer(
	program_data: &ProgramData,
	dd_type: &DDTypeUniform,
) -> FrameDescriptor { unsafe {
	let mut dummy = Vec::with_capacity(dd_type.size);
	dummy.resize(dd_type.size, 0u8);
	let buffer = GO_Uniform::new(
		program_data,
		&dummy,
	);
	FrameDescriptor::Uniform(DescriptorUniform {
		binding_id: dd_type.binding,
		buffer,
		size: dd_type.size,
	})
}}

fn create_image_buffer(
	program_data: &ProgramData,
	cmd_buffer: &vk::CommandBuffer,
	dd_type: &DDTypeImage,
) -> FrameDescriptor { unsafe {
	let image = GO_Image::new(
		program_data,
		&dd_type.file_abs,
	);
	image.transfer(&program_data.device, cmd_buffer);
	FrameDescriptor::Image(DescriptorImage {
		binding_id: dd_type.binding,
		image,
	})
}}

fn create_image_array_buffer(
	program_data: &ProgramData,
	cmd_buffer: &vk::CommandBuffer,
	dd_type: &DDTypeImageArray,
) -> FrameDescriptor { unsafe {
	let image_array = GO_ImageArray::new(
		program_data,
		&dd_type.ias,
	);
	image_array.transfer(&program_data.device, cmd_buffer);
	FrameDescriptor::ImageArray(DescriptorImageArray {
		binding_id: dd_type.binding,
		image_array,
	})
}}

fn create_write_uniform(
	uniform: &mut DescriptorUniform,
	descriptor_set: vk::DescriptorSet,
) -> WriteDS {
	let buffer = match &uniform.buffer.buffer.buffer {
		BufferType::Buffer(buffer) => buffer,
		BufferType::Image(_) => unreachable!(),
	};
	let info = vk::DescriptorBufferInfo::builder()
		.buffer(buffer.buffer)
		.offset(0)
		.range(uniform.size as u64)
		.build();
	let mut ds_info = Vec::with_capacity(1);
	ds_info.push(info);
	let write = vk::WriteDescriptorSet::builder()
		.dst_set(descriptor_set)
		.dst_binding(uniform.binding_id.0)
		.dst_array_element(0)
		.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
		.buffer_info(&ds_info)
		.build();
	WriteDS {
		write,
		info: WriteDSInfo::Uniform(ds_info),
	}
}

fn create_write_image(
	image: &mut DescriptorImage,
	descriptor_set: vk::DescriptorSet,
) -> WriteDS {
	let buffer_image = match &image.image.image_buffer.buffer {
		BufferType::Buffer(_) => unreachable!(),
		BufferType::Image(image) => image,
	};
	let info = vk::DescriptorImageInfo::builder()
		.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
		.image_view(buffer_image.image_view)
		.sampler(buffer_image.image_sampler)
		.build();
	let mut ds_info = Vec::with_capacity(1);
	ds_info.push(info);
	let write = vk::WriteDescriptorSet::builder()
		.dst_set(descriptor_set)
		.dst_binding(image.binding_id.0)
		.dst_array_element(0)
		.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
		.image_info(&ds_info)
		.build();
	WriteDS {
		write,
		info: WriteDSInfo::Image(ds_info),
	}
}

fn create_write_image_array(
	image_array: &mut DescriptorImageArray,
	descriptor_set: vk::DescriptorSet,
) -> WriteDS {
	let buffer_image = match &image_array.image_array.image_buffer.buffer {
		BufferType::Buffer(_) => unreachable!(),
		BufferType::Image(image) => image,
	};
	let info = vk::DescriptorImageInfo::builder()
		.image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
		.image_view(buffer_image.image_view)
		.sampler(buffer_image.image_sampler)
		.build();
	let mut ds_info = Vec::with_capacity(1);
	ds_info.push(info);
	let write = vk::WriteDescriptorSet::builder()
		.dst_set(descriptor_set)
		.dst_binding(image_array.binding_id.0)
		.dst_array_element(0)
		.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
		.image_info(&ds_info)
		.build();
	WriteDS {
		write,
		info: WriteDSInfo::Image(ds_info),
	}
}