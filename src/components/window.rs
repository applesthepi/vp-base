use ash::vk;
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::LogicalSize};

pub struct Window {
	pub event_loop: EventLoop<()>,
	pub extent: vk::Extent2D,
	pub window: winit::window::Window,
}

impl Window {
	pub fn new(
		title: &str,
	) -> Self {
		let extent = vk::Extent2D {
			width: 1280,
			height: 720,
		};
		let event_loop = EventLoop::new();
		let window = WindowBuilder::new()
			.with_title(title)
			.with_inner_size(LogicalSize::new(
				extent.width,
				extent.height,
			))
			.build(&event_loop)
			.unwrap();
		Self {
			event_loop,
			extent,
			window,
		}
	}
}