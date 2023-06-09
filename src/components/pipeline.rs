use std::{fs::File, io::Read, sync::Arc, ffi::CStr, cell::RefCell};

use ash::{util::read_spv, vk::{self, ShaderModule, ShaderStageFlags}};
use shaderc::{Compiler, CompileOptions, ShaderKind};

use crate::{Device, BlockState};

// pub trait Pipeline {
// 	fn get_viewport(&self) -> [vk::Viewport; 1];
// 	fn get_scissor(&self) -> [vk::Rect2D; 1];
	
// 	fn destroy_block_state_memory(
// 		&mut self,
// 		device: &Arc<Device>,
// 	);
	
// 	fn destroy_pipeline(
// 		&mut self,
// 		device: &Arc<Device>,
// 	);

// 	fn get_pipeline(&self) -> vk::Pipeline;
// 	fn get_pipeline_layout(&self) -> vk::PipelineLayout;
// 	fn get_block(&self) -> Arc<BlockState>;
	
// 	fn update_blocks(
// 		&mut self,
// 		device: &Device,
// 		extent: &vk::Extent2D,
// 		frame: usize,
// 	);

// 	fn object_binding_set(
// 	) -> Vec<(u32, u32)> where Self: Sized;
// }

pub struct ShaderLoader {
	pub compiler: Compiler,
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