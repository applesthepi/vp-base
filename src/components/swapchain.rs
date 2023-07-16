use ash::{vk, extensions::khr};
use glfw::Context;

use crate::{Device, Window, Instance, Surface};

pub struct Swapchain {
	pub swapchain_loader: khr::Swapchain,
	pub swapchain: vk::SwapchainKHR,
	pub present_queue: vk::Queue,
	pub surface_format: vk::SurfaceFormatKHR,
}

impl Swapchain {
	pub fn new(
		instance: &Instance,
		window: &mut Window,
		surface: &Surface,
		device: &Device,
	) -> Self { unsafe {
		window.window.make_current();
		let present_queue = device.device.get_device_queue(
			device.queue_family_index[0],
			0
		);
		let surface_format =
			device.surface_loader
			.get_physical_device_surface_formats(
				device.physical_device,
				surface.surface,
			).unwrap()[0];
		let surface_capabilities =
			device.surface_loader
			.get_physical_device_surface_capabilities(
				device.physical_device,
				surface.surface,
			).unwrap();
		window.extent = match surface_capabilities.current_extent.width {
			u32::MAX => window.extent,
			_ => surface_capabilities.current_extent,
		};

		let mut desired_image_count = surface_capabilities.min_image_count + 1;
		if surface_capabilities.max_image_count > 0 &&
			desired_image_count > surface_capabilities.max_image_count {
			desired_image_count = surface_capabilities.max_image_count;
		}
		let pre_transform = if surface_capabilities
			.supported_transforms
			.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
				vk::SurfaceTransformFlagsKHR::IDENTITY
			} else {
				surface_capabilities.current_transform
			};
		let present_modes = device.surface_loader
			.get_physical_device_surface_present_modes(device.physical_device, surface.surface)
			.unwrap();
		let present_mode = present_modes
			.iter().cloned().find(
				|&mode|
				mode == vk::PresentModeKHR::MAILBOX
			).unwrap_or(vk::PresentModeKHR::FIFO);
		let swapchain_loader = khr::Swapchain::new(&instance.instance, &device.device);
		let swapchain_info =
			vk::SwapchainCreateInfoKHR::builder()
			.surface(surface.surface)
			.min_image_count(desired_image_count)
			.image_color_space(surface_format.color_space)
			.image_format(surface_format.format)
			.image_extent(window.extent)
			.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
			.pre_transform(pre_transform)
			.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
			.present_mode(present_mode)
			.clipped(true)
			.image_array_layers(1);
		let swapchain = swapchain_loader.create_swapchain(
			&swapchain_info,
			None,
		).unwrap();	
		Self {
			swapchain_loader,
			swapchain,
			present_queue,
			surface_format,
		}
	}}
}