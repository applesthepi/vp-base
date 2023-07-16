use std::sync::mpsc::Receiver;

use ash::vk;
use glfw::{Context, WindowEvent, Glfw};

pub struct Window {
	pub extent: vk::Extent2D,
	pub glfw: Glfw,
	pub window: glfw::Window,
	pub events: Receiver<(f64, WindowEvent)>,
}

impl Window {
	pub fn new(
		title: &str,
	) -> Self {
		let extent = vk::Extent2D {
			width: 1280,
			height: 720,
		};
		let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
		glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
		let (mut window, events) = glfw
			.create_window(extent.width, extent.height, title, glfw::WindowMode::Windowed)
			.expect("failed to create glfw window");
		window.set_key_polling(true);
		assert!(glfw.vulkan_supported());
		Self {
			extent,
			glfw,
			window,
			events,
		}
	}
}