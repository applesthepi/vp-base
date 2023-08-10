use std::fs::{File, self};

use ash::vk;
use bytemuck::bytes_of;
use nalgebra::{Vector2, vector};
use serde::__private::de;

use crate::{BufferGO, Instance, Device, RequirementType, ProgramData, ImageArrayState};

// #[derive(Clone)]
#[allow(non_camel_case_types)]
/// Vertex, index, and indirect buffers with gpu only memory (nothing cached).
pub struct GO_ImageArray {
	pub image_staging_buffer: Option<BufferGO>,
	pub image_buffer: BufferGO,
	pub image_size: Vector2<u32>,
	pub image_layers: u32,
}

impl GO_ImageArray {
	pub fn new(
		program_data: &ProgramData,
		ias: &ImageArrayState,
	) -> GO_ImageArray { unsafe {
		let (
			image_data,
			image_size,
			image_layers,
		) = GO_ImageArray::load_disk(ias);
		let mut image_staging_buffer = BufferGO::new::<u8>(
			program_data,
			RequirementType::Buffer(image_data.len(), vk::BufferUsageFlags::TRANSFER_SRC),
		);
		image_staging_buffer.update(program_data, &image_data);
		let image_buffer = BufferGO::new::<u8>(
			program_data,
			RequirementType::Image(vk::Extent2D::builder().width(image_size.x).height(image_size.y).build(), Some(image_layers)),
		);
		GO_ImageArray {
			image_staging_buffer: Some(image_staging_buffer),
			image_buffer,
			image_size,
			image_layers,
		}
	}}

	pub fn update_image_array(
		&mut self,
		program_data: &ProgramData,
		ias: &ImageArrayState,
	) { unsafe {
		let (
			image_data,
			image_size,
			image_layers,
		) = GO_ImageArray::load_disk(ias);
		self.image_size = image_size;
		self.image_layers = image_layers;
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
		ias: &ImageArrayState,
	) -> (Vec<u8>, Vector2<u32>, u32) {
		let mut files: Vec<(Vec<u8>, Vector2<u32>)> = Vec::with_capacity(1024);
		for texture in ias.textures.iter() {
			let mut path = String::with_capacity(ias.path.len() + 1 + texture.len() + 4);
			path += &ias.path;
			path += "/";
			path += &texture;
			path += ".png";
			files.push(GO_ImageArray::load_disk_single(&path));
		}
		// let paths = fs::read_dir(ias.path).expect("directory does not exist");
		// for path in paths {
		// 	let path = path.unwrap().path().display().to_string();
		// 	files.push(GO_ImageArray::load_disk_single(&path));
		// }
		let mut buffer: Vec<u8> = Vec::with_capacity(4096 * files.len());
		for file in files.iter() {
			assert!(file.1.x == 32 && file.1.y == 32);
			assert!(file.0.len() == 4096);
			buffer.extend_from_slice(file.0.as_slice());
		}
		(buffer, vector![32, 32], files.len() as u32)
	}

	fn load_disk_single(
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
				.layer_count(self.image_layers)
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
		let mut regions: Vec<vk::BufferImageCopy> = Vec::with_capacity(self.image_layers as usize);
		for layer in 0..self.image_layers {
			let region = vk::BufferImageCopy::builder()
				.buffer_offset((layer * 4096) as u64)
				.buffer_row_length(0)
				.buffer_image_height(0)
				.image_subresource(vk::ImageSubresourceLayers::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.mip_level(0)
					.base_array_layer(layer)
					.layer_count(1)
					.build())
				.image_offset(vk::Offset3D::builder().x(0).y(0).z(0).build())
				.image_extent(vk::Extent3D::builder().depth(1).width(self.image_size.x).height(self.image_size.y).build())
				.build();
			regions.push(region);
		}
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
			&regions,
		);
	}}
}