use std::marker::PhantomData;

use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{Window, Instance};

pub struct Surface {
	pub surface: vk::SurfaceKHR,
}

impl Surface {
	pub fn new(
		instance: &Instance,
		window: &Window,
	) -> Self { unsafe {
		let surface = ash_window::create_surface(
			&instance.entry,
			&instance.instance,
			window.window.raw_display_handle(),
			window.window.raw_window_handle(),
			None,
		).unwrap();
		Self {
			surface,
		}
	}}
}