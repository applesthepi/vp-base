use std::{fs::File, io::Read, sync::Arc, ffi::CStr, cell::RefCell};

use ash::{util::read_spv, vk::{self, ShaderModule, ShaderStageFlags}};
use shaderc::{Compiler, CompileOptions, ShaderKind};

use crate::{Device, BlockState};

pub trait Pipeline {
	fn get_viewport(&self) -> [vk::Viewport; 1];
	fn get_scissor(&self) -> [vk::Rect2D; 1];
	fn get_pipeline(&self) -> vk::Pipeline;
	// fn get_blocks(&self) -> Vec<BlockState>;
	fn bind_blocks(
		&self,
		device: &Device,
		command_buffer: &vk::CommandBuffer,
		frame: usize,
	);
	fn destroy_set_layouts(
		&self,
		device: &Device,
	);
}

pub struct ShaderLoader {
	compiler: Compiler,
}

impl ShaderLoader {
	pub fn new(

	) -> Self {
		let compiler = Compiler::new().unwrap();
		let shader_loader = ShaderLoader {
			compiler,
		};
		shader_loader
	}
}

pub fn create_stage_infos(
	shader_stages: &[(ShaderModule, ShaderStageFlags)],
) -> Vec<vk::PipelineShaderStageCreateInfo> { unsafe {
	let name = CStr::from_bytes_with_nul_unchecked(b"main\0");
	shader_stages.iter().map(
		|(shader_module, shader_stage)|
		vk::PipelineShaderStageCreateInfo::builder()
			.module(*shader_module)
			.name(name)
			.stage(*shader_stage)
			.build()
	).collect()
}}