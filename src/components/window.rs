use std::marker::PhantomData;

use ash::vk;
use winit::{event_loop::EventLoop, window::WindowBuilder, dpi::LogicalSize};

pub struct Window {
	pub extent: vk::Extent2D,
	pub window: winit::window::Window,
}

impl Window {
	pub fn new(
		title: &str,
		event_loop: &EventLoop<()>,
	) -> Self {
		let extent = vk::Extent2D {
			width: 1280,
			height: 720,
		};
		let window = WindowBuilder::new()
			.with_title(title)
			.with_inner_size(LogicalSize::new(
				extent.width as f64,
				extent.height as f64,
			))
			.build(event_loop)
			.unwrap();
		Self {
			extent,
			window,
		}
	}
}