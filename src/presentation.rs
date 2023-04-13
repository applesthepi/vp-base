use ash::vk;

use crate::{Swapchain, Device, Instance, Window};

pub fn create_presentation_images(
	device: &Device,
	swapchain: &Swapchain,
) -> (Vec<vk::Image>, Vec<vk::ImageView>) { unsafe {
	let images = swapchain.swapchain_loader.get_swapchain_images(
		swapchain.swapchain,
	).unwrap();
	let mut image_views: Vec<vk::ImageView> = Vec::with_capacity(images.len());
	for image in images.iter() {
		let image_view_info = vk::ImageViewCreateInfo::builder()
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(swapchain.surface_format.format)
			.components(vk::ComponentMapping {
				r: vk::ComponentSwizzle::R,
				g: vk::ComponentSwizzle::G,
				b: vk::ComponentSwizzle::B,
				a: vk::ComponentSwizzle::A,
			})
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.image(*image)
			.build();
		image_views.push(device.device.create_image_view(
			&image_view_info,
			None
		).unwrap());
	}
	(images, image_views)
}}

pub fn create_depth_image(
	instance: &Instance,
	device: &Device,
	window: &Window,
) -> () { unsafe {
	let device_memory_properties = instance.instance.get_physical_device_memory_properties(
		device.physical_device
	);
	let image_info = vk::ImageCreateInfo::builder()
		.image_type(vk::ImageType::TYPE_2D)
		.format(vk::Format::D16_UNORM)
		.extent(window.extent.into())
		.mip_levels(1)
		.array_layers(1)
		.samples(vk::SampleCountFlags::TYPE_1)
		.tiling(vk::ImageTiling::OPTIMAL)
		.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
		.sharing_mode(vk::SharingMode::EXCLUSIVE)
		.build();
	let image = device.device.create_image(
		&image_info,
		None
	).unwrap();
	let memory_requirements = device.device.get_image_memory_requirements(
		image,
	);
	let memory_index: u32 = device_memory_properties.memory_types[..device_memory_properties.memory_type_count as _]
		.iter().enumerate().find(
			|(index, memory_type)| {
				(1 << index) & memory_requirements.memory_type_bits != 0 &&
				memory_type.property_flags & vk::MemoryPropertyFlags::DEVICE_LOCAL == vk::MemoryPropertyFlags::DEVICE_LOCAL
			}
		).map(
			|(index, _)| {
				index as _
			}
		).unwrap();
	let memory_alloc_info = vk::MemoryAllocateInfo::builder()
		.allocation_size(memory_requirements.size)
		.memory_type_index(memory_index)
		.build();
	let memory_alloc = device.device.allocate_memory(
		&memory_alloc_info,
		None,
	).unwrap();
	device.device.bind_image_memory(
		image,
		memory_alloc,
		0,
	).unwrap();
}}