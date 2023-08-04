
use std::mem::size_of;

use ash::vk;
use bytemuck::Pod;

use crate::{Vertex, Device, VertexBuffer, BufferGO, Instance, IndexBuffer, RequirementType, program_data, ProgramData, BufferType};

#[allow(non_camel_case_types)]
pub struct GO_Uniform {
	pub buffer: BufferGO,
}

impl GO_Uniform {
	pub fn new(
		program_data: &ProgramData,
		data: &[u8],
	) -> Self {
		let mut buffer = BufferGO::new::<u8>(
			program_data,
			RequirementType::Buffer(
				data.len(),
				vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
			),
		);
		buffer.update(program_data, data);
		Self {
			buffer,
		}
	}

	pub fn update(
		&mut self,
		program_data: &ProgramData,
		data: &[u8],
	) {
		self.buffer.update(program_data, data);
	}
}