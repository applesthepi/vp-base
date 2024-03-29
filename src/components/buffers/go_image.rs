use std::fs::File;

use ash::vk;
use bytemuck::bytes_of;
use nalgebra::{Vector2, vector};
use serde::__private::de;

use crate::{BufferGO, Instance, Device, RequirementType, ProgramData};

// #[derive(Clone)]
#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_Image {
	pub image_staging_buffer: Option<BufferGO>,
	pub image_buffer: BufferGO,
	pub image_size: Vector2<u32>,
	pub file_abs: String,
}

impl GO_Image {
	pub fn new(
		program_data: &ProgramData,
		file_name_abs: &str,
	) -> GO_Image { unsafe {
		let (
			image_data,
			image_size,
		) = GO_Image::load_disk(file_name_abs);
		let mut image_staging_buffer = BufferGO::new::<u8>(
			program_data,
			RequirementType::Buffer(image_data.len(), vk::BufferUsageFlags::TRANSFER_SRC),
		);
		image_staging_buffer.update(program_data, &image_data);
		let image_buffer = BufferGO::new::<u8>(
			program_data,
			RequirementType::Image(vk::Extent2D::builder().width(image_size.x).height(image_size.y).build(), None),
		);
		GO_Image {
			image_staging_buffer: Some(image_staging_buffer),
			image_buffer,
			image_size,
			file_abs: file_name_abs.to_string(),
		}
	}}

	pub fn update_image(
		&mut self,
		program_data: &ProgramData,
		file_name_abs: &str,
	) { unsafe {
		self.file_abs = file_name_abs.to_string();
		let (
			image_data,
			image_size,
		) = GO_Image::load_disk(file_name_abs);
		self.image_size = image_size;
		if let Some(image_staging_buffer) = &mut self.image_staging_buffer {
			image_staging_buffer.update(program_data, &image_data);
		} else {
			let mut image_staging_buffer = BufferGO::new::<u8>(
				program_data,
				RequirementType::Buffer(image_data.len(), vk::BufferUsageFlags::TRANSFER_SRC),
			);
			image_staging_buffer.update(program_data, &image_data);
		}
	}}

	pub fn transfer(
		&self,
		device: &Device,
		command_buffer: &vk::CommandBuffer,
	) {
		self.image_barrier(
			device,
			command_buffer,
			vk::ImageLayout::UNDEFINED,
			vk::ImageLayout::TRANSFER_DST_OPTIMAL,
		);
		self.gpu_copy_image(device, command_buffer);
		self.image_barrier(
			device,
			command_buffer,
			vk::ImageLayout::TRANSFER_DST_OPTIMAL,
			vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
		);
	}

	fn load_disk(
		file_name_abs: &str,
	) -> (Vec<u8>, Vector2<u32>) {
		let mut decoder = png::Decoder::new(File::open(file_name_abs).expect(
			&format!("invalid image abs path \"{}\"", file_name_abs),
		));
		let header_info = decoder.read_header_info().unwrap();
		if header_info.bytes_per_pixel() != 4 {
			panic!("must have 4 bytes per pixel (RGBA) for file abs \"{}\"", file_name_abs);
		}
		let image_size = vector![header_info.width, header_info.height];
		drop(header_info);
		let mut reader = decoder.read_info().unwrap();
		let mut png_buffer = vec![0u8; reader.output_buffer_size()];
		while let Ok(_) = reader.next_frame(&mut png_buffer) {}
		(png_buffer, image_size)
	}

	fn image_barrier(
		&self,
		device: &Device,
		command_buffer: &vk::CommandBuffer,
		old_layout: vk::ImageLayout,
		new_layout: vk::ImageLayout,
	) { unsafe {
		let (
			src_access_mask,
			dst_access_mask,
			src_stage_mask,
			dst_stage_mask,
		 ) = match old_layout {
			vk::ImageLayout::UNDEFINED => {
				(
					vk::AccessFlags::empty(),
					vk::AccessFlags::TRANSFER_WRITE,
					vk::PipelineStageFlags::TOP_OF_PIPE,
					vk::PipelineStageFlags::TRANSFER,
				)
			},
			vk::ImageLayout::TRANSFER_DST_OPTIMAL => {
				(
					vk::AccessFlags::TRANSFER_WRITE,
					vk::AccessFlags::SHADER_READ,
					vk::PipelineStageFlags::TRANSFER,
					vk::PipelineStageFlags::VERTEX_SHADER,
				)
			},
			_ => unimplemented!()
		};
		let memory_barrier = vk::ImageMemoryBarrier::builder()
			.old_layout(old_layout)
			.new_layout(new_layout)
			.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.image(match &self.image_buffer.buffer {
				crate::BufferType::Buffer(_) => { unreachable!(); },
				crate::BufferType::Image(image) => { image.image },
			})
			.subresource_range(vk::ImageSubresourceRange::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.base_mip_level(0)
				.level_count(1)
				.base_array_layer(0)
				.layer_count(1)
				.build())
			.src_access_mask(src_access_mask)
			.dst_access_mask(dst_access_mask)
			.build();
		device.device.cmd_pipeline_barrier(
			*command_buffer,
			src_stage_mask,
			dst_stage_mask,
			vk::DependencyFlags::empty(),
			&[],
			&[],
			&[memory_barrier],
		);
	}}

	fn gpu_copy_image(
		&self,
		device: &Device,
		command_buffer: &vk::CommandBuffer,
	) { unsafe {
		let region = vk::BufferImageCopy::builder()
			.buffer_offset(0)
			.buffer_row_length(0)
			.buffer_image_height(0)
			.image_subresource(vk::ImageSubresourceLayers::builder()
				.aspect_mask(vk::ImageAspectFlags::COLOR)
				.mip_level(0)
				.base_array_layer(0)
				.layer_count(1)
				.build())
			.image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
			.image_extent(vk::Extent3D::builder().depth(1).width(self.image_size.x).height(self.image_size.y).build())
			.build();
		device.device.cmd_copy_buffer_to_image(
			*command_buffer,
			match &self.image_staging_buffer.as_ref().unwrap_unchecked().buffer {
				crate::BufferType::Buffer(buffer) => {
					buffer.buffer
				},
				crate::BufferType::Image(_) => { unreachable!(); },
			},
			match &self.image_buffer.buffer {
				crate::BufferType::Buffer(_) => { unreachable!(); },
				crate::BufferType::Image(image) => {
					image.image
				},
			},
			vk::ImageLayout::TRANSFER_DST_OPTIMAL,
			&[region],
		);
	}}
}