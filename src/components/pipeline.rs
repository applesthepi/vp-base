use std::{fs::File, io::Read, sync::Arc, ffi::CStr};

use ash::{util::read_spv, vk::{self, ShaderModule, ShaderStageFlags}};
use shaderc::{Compiler, CompileOptions, ShaderKind};

use crate::Device;

pub trait Pipeline {
	fn get_viewport(&self) -> [vk::Viewport; 1];
	fn get_scissor(&self) -> [vk::Rect2D; 1];
	fn get_pipeline(&self) -> vk::Pipeline;
}

pub struct ShaderLoader<'a> {
	compiler: Compiler,
	options: CompileOptions<'a>,
}

impl<'a> ShaderLoader<'a> {
	pub fn new(

	) -> Arc<Self> {
		let compiler = Compiler::new().unwrap();
		let options = CompileOptions::new().unwrap();
		let shader_loader = ShaderLoader {
			compiler,
			options,
		};
		Arc::new(shader_loader)
	}
}

pub fn load_shader(
	device: &Device,
	shader_loader: &Arc<ShaderLoader>,
	shader_kind: ShaderKind,
	name: &str,
) -> ShaderModule { unsafe {
	let glsl_path = ("res/shaders/".to_string() + name) + match shader_kind {
		ShaderKind::Vertex => ".vert",
		ShaderKind::Fragment => ".frag",
		ShaderKind::Compute => ".comp",
		_ => { panic!("not impl"); }
	};
	let glsl_path = glsl_path.as_str();
	let spv_path = ("res/shaders/".to_string() + name) + ".spv";
	let spv_path = spv_path.as_str();
	let mut file = File::open(glsl_path).expect(
		format!("shader \"{}\" does not exist", glsl_path).as_str()
	);
	let mut text: String = String::with_capacity(1024);
	file.read_to_string(&mut text).unwrap();
	let binary_artifact = shader_loader.compiler.compile_into_spirv(
		text.as_str(),
		shader_kind,
		glsl_path, "main",
		Some(&shader_loader.options),
	).expect(format!("failed to compile \"{}\"", glsl_path).as_str());
	debug_assert_eq!(Some(&0x07230203), binary_artifact.as_binary().first());
	// let text_artifact = shader_loader.compiler.compile_into_spirv_assembly(
	// 	text.as_str(),
	// 	shader_kind,
	// 	glsl_path, "main",
	// 	Some(&shader_loader.options),
	// ).expect(format!("failed to compile \"{}\"", glsl_path).as_str());
	// debug_assert!(text_artifact.as_text().starts_with("; SPIR-V\n"));
	// let mut spv_file = File::open(spv_path).unwrap();
	// let spv_text = read_spv(&mut spv_file).unwrap();

	let spv_text = binary_artifact.as_binary();

	let shader_info = vk::ShaderModuleCreateInfo::builder()
		.code(spv_text)
		.build();
	device.device.create_shader_module(
		&shader_info,
		None,
	).unwrap()
}}

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