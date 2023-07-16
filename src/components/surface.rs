use std::{marker::PhantomData, ptr};

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
		// let surface = ash_window::create_surface(
		// 	&instance.entry,
		// 	&instance.instance,
		// 	window.window.raw_display_handle(),
		// 	window.window.raw_window_handle(),
		// 	None,
		// ).unwrap();

		let mut surface: std::mem::MaybeUninit<vk::SurfaceKHR> = std::mem::MaybeUninit::uninit();
		if window.window.create_window_surface(instance.instance.handle(), ptr::null(), surface.as_mut_ptr())
			!= vk::Result::SUCCESS
		{
			panic!("Failed to create GLFW window surface.");
		}
		Self {
			surface: surface.assume_init(),
		}
	}}
}