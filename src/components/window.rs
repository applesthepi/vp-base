use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::LogicalSize};

use crate::Instance;

pub struct Window {
	pub event_loop: EventLoop<()>,
	pub surface: vk::SurfaceKHR,
}

impl Window {
	pub fn new(
		title: &str,
		instance: &Instance,
	) -> Self { unsafe {
		let event_loop = EventLoop::new();
		let window = WindowBuilder::new()
			.with_title(title)
			.with_inner_size(LogicalSize::new(
				1280.0,
				720.0,
			))
			.build(&event_loop)
			.unwrap();
		let surface = ash_window::create_surface(
			&instance.entry,
			&instance.instance,
			window.raw_display_handle(),
			window.raw_window_handle(),
			None,
		).unwrap();
		Self {
			event_loop,
			surface,
		}
	}}
}