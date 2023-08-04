use std::{sync::Arc, fs::File, io::Read};

use ash::vk::{self, DeviceMemory};
use shaderc::{ShaderKind, CompileOptions};

use crate::{Window, Instance, Surface, Device, Swapchain, RenderPass, DescriptorPool, CommandPool, CommandBuffer, ShaderLoader};

#[derive(Clone)]
pub struct ProgramData {
	pub allocator: Arc<Option<Arc<vk_mem::Allocator>>>,
	pub window: Arc<Window>,
	pub instance: Arc<Instance>,
	pub surface: Arc<Surface>,
	pub device: Arc<Device>,
	pub swapchain: Arc<Swapchain>,
	pub render_pass: Arc<RenderPass>,
	pub descriptor_pool: Arc<DescriptorPool>,
	pub command_pool: Arc<CommandPool>,
	pub command_buffer_setup: Arc<CommandBuffer>,
	pub command_buffer_draw: Arc<CommandBuffer>,
	pub shader_loader: Arc<ShaderLoader>,
	pub frame_count: usize,
}

impl ProgramData {
	pub fn get_allocator(
		&self,
	) -> &vk_mem::Allocator {
		&self.allocator.as_ref().as_ref().unwrap()
	}

	pub fn load_shader(
		&self,
		shader_kind: ShaderKind,
		name: &str,
	) -> vk::ShaderModule { unsafe {
		let options = CompileOptions::new().unwrap();
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
		let binary_artifact = self.shader_loader.compiler.compile_into_spirv(
			text.as_str(),
			shader_kind,
			glsl_path, "main",
			Some(&options),
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
		self.device.device.create_shader_module(
			&shader_info,
			None,
		).unwrap()
	}}

	pub fn create_allocator(
		instance: ash::Instance,
		device: ash::Device,
		physical_device: vk::PhysicalDevice,
		frame_count: usize,
	) -> Arc<vk_mem::Allocator> { unsafe {
		let allocator_info = vk_mem::AllocatorCreateInfo {
			physical_device,
			device,
			instance,
			flags: vk_mem::AllocatorCreateFlags::NONE,
			preferred_large_heap_block_size: 0,
			frame_in_use_count: frame_count as u32,
			heap_size_limits: None,
		};
		Arc::new(
			vk_mem::Allocator::new(
				&allocator_info,
			).expect("failed to create allocator")
		)
	}}
}