use std::mem::{align_of, size_of};

use ash::{vk::{self, DeviceMemory, MemoryPropertyFlags}, util::Align};
use bytemuck::{Pod, cast_slice};
use vk_mem::{AllocationCreateInfo, MemoryUsage, AllocationCreateFlags, Allocation};

use crate::{Device, Instance, ProgramData};

#[derive(Clone, Debug)]
pub enum RequirementType {
	Buffer(usize, vk::BufferUsageFlags),
	Image(vk::Extent2D),
}

#[derive(Clone, Debug)]
pub enum BufferType {
	Buffer(BufferTypeBuffer),
	Image(BufferTypeImage),
}

#[derive(Clone, Debug)]
pub struct BufferTypeBuffer {
	pub buffer: vk::Buffer,
	pub buffer_offset: usize,
	pub buffer_allocation: Allocation,
	pub mapped: *mut u8,
}

#[derive(Clone, Debug)]
pub struct BufferTypeImage {
	pub image: vk::Image,
	pub image_view: vk::ImageView,
	pub image_sampler: vk::Sampler,
	pub image_allocation: Allocation,
}

#[derive(Clone)]
pub struct BufferGOMemory(pub vk::Buffer, pub vk::DeviceMemory);

// TODO: staging for host coherent and local for device read only
pub struct BufferGO {
	pub count: usize,
	pub capacity: usize,
	pub buffer: BufferType,
	pub requirement_type: RequirementType,
}

impl BufferGO {
	pub fn new<T>(
		program_data: &ProgramData,
		requirement_type: RequirementType,
	) -> Self
	where T: Default + Copy + Clone {
		let buffer = allocate(
			program_data,
			&requirement_type,
		);
		match requirement_type {
			RequirementType::Buffer(size, _) => {
				Self {
					count: 0,
					capacity: size,
					buffer,
					requirement_type,
				}
			},
			RequirementType::Image(extent) => {
				Self {
					count: 0,
					capacity: (extent.width * extent.height * 4) as usize,
					buffer,
					requirement_type,
				}
			}
		}
	}

	pub fn update<T>(
		&mut self,
		program_data: &ProgramData,
		data: &[T],
	) where
	T: Default + Copy + Clone + Pod { unsafe {
		if data.is_empty() {
			self.count = 0;
			return;
		}
		let data: &[u8] = cast_slice(data);
		if self.capacity < data.len() {
			let mut n_capacity: usize = self.capacity.max(data.len());
			while n_capacity < data.len() {
				n_capacity *= 2;
			}
			match &self.buffer {
				BufferType::Buffer(buffer) => {
					program_data.get_allocator().destroy_buffer(
						buffer.buffer,
						&buffer.buffer_allocation,
					).expect("failed to destroy buffer");
				},
				BufferType::Image(image) => {
					program_data.device.device.destroy_sampler(
						image.image_sampler,
						None,
					);
					program_data.device.device.destroy_image_view(
						image.image_view,
						None,
					);
					program_data.get_allocator().destroy_image(
						image.image,
						&image.image_allocation,
					).expect("failed to destroy image");
				},
			}
			let buffer = allocate(
				program_data,
				&self.requirement_type,
			);
			self.buffer = buffer;
			self.capacity = n_capacity;
		}
		self.count = data.len();
		match &self.buffer {
			BufferType::Buffer(buffer) => {
				buffer.mapped.copy_from(data.as_ptr(), data.len());
			},
			BufferType::Image(image) => { unimplemented!(); },
		}
	}}
}

fn allocate(
	program_data: &ProgramData,
	requriement_type: &RequirementType,
) -> BufferType { unsafe {
	match requriement_type {
		RequirementType::Buffer(size, usage_flags) => {
			let allocation_info = AllocationCreateInfo {
				usage: MemoryUsage::CpuToGpu,
				flags: AllocationCreateFlags::MAPPED,
				required_flags: MemoryPropertyFlags::empty(),
				preferred_flags: MemoryPropertyFlags::empty(),
				memory_type_bits: 0,
				pool: None,
				user_data: None,
			};
			let buffer_info = vk::BufferCreateInfo::builder()
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				.size(*size as u64)
				.usage(*usage_flags)
				.build();
			let (
				buffer,
				buffer_allocation,
				buffer_allocation_info,
			) = program_data.get_allocator().create_buffer(
				&buffer_info,
				&allocation_info,
			).expect("failed to create buffer");
			let mapped = buffer_allocation_info.get_mapped_data();
			let buffer_offset = buffer_allocation_info.get_offset();
			BufferType::Buffer(BufferTypeBuffer {
				buffer,
				buffer_offset,
				buffer_allocation,
				mapped,
			})
		},
		RequirementType::Image(extent) => {
			let allocation_info = AllocationCreateInfo {
				usage: MemoryUsage::GpuOnly,
				flags: AllocationCreateFlags::empty(),
				required_flags: MemoryPropertyFlags::empty(),
				preferred_flags: MemoryPropertyFlags::empty(),
				memory_type_bits: 0,
				pool: None,
				user_data: None,
			};
			let image_info = vk::ImageCreateInfo::builder()
				.image_type(vk::ImageType::TYPE_2D)
				.extent(vk::Extent3D::builder().depth(1).width(extent.width).height(extent.height).build())
				.mip_levels(1)
				.array_layers(1)
				.format(vk::Format::R8G8B8A8_SRGB)
				.tiling(vk::ImageTiling::OPTIMAL)
				.initial_layout(vk::ImageLayout::UNDEFINED)
				.usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				.samples(vk::SampleCountFlags::TYPE_1)
				.flags(vk::ImageCreateFlags::empty())
				.build();
			let (
				image,
				image_allocation,
				image_allocation_info,
			) = program_data.get_allocator().create_image(
				&image_info,
				&allocation_info,
			).expect("failed to create image");

			// VIEW & SAMPLER

			let image_view_info = vk::ImageViewCreateInfo::builder()
				.image(image)
				.view_type(vk::ImageViewType::TYPE_2D)
				.format(vk::Format::R8G8B8A8_SRGB)
				.subresource_range(vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build())
				.build();
			let image_view = program_data.device.device.create_image_view(
				&image_view_info,
				None,
			).expect("failed to create image view");
			let image_sampler_info = vk::SamplerCreateInfo::builder()
				.mag_filter(vk::Filter::NEAREST)
				.min_filter(vk::Filter::NEAREST)
				.address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
				.address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
				.address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
				.anisotropy_enable(false)
				.unnormalized_coordinates(false)
				.compare_enable(false)
				.compare_op(vk::CompareOp::ALWAYS)
				.mipmap_mode(vk::SamplerMipmapMode::LINEAR)
				.mip_lod_bias(0.0)
				.min_lod(0.0)
				.max_lod(0.0)
				.build();
			let image_sampler = program_data.device.device.create_sampler(
				&image_sampler_info,
				None,
			).expect("failed to create image sampler");

			BufferType::Image(BufferTypeImage {
				image,
				image_view,
				image_sampler,
				image_allocation,
			})
		},
	}
}}