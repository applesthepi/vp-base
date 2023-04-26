use std::ffi::c_char;

use ash::{vk, extensions::khr};

use crate::{Instance, Surface};

const DEVICE_EXTENSIONS: [*const c_char; 1] = [
	khr::Swapchain::name().as_ptr(),
];
const PRIORITIES: [f32; 1] = [
	1.0,
];

pub struct Device {
	pub device: ash::Device,
	pub physical_device: vk::PhysicalDevice,
	pub queue_family_index: u32,
	pub surface_loader: khr::Surface,
	pub physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
	pub surface_format: vk::SurfaceFormatKHR,
}

impl Device {
	pub fn new(
		instance: &Instance,
		surface: &Surface,
	) -> Self { unsafe {
		let features: vk::PhysicalDeviceFeatures =
			vk::PhysicalDeviceFeatures {
				shader_clip_distance: 1,
				..Default::default()
			};
		let pdevices =
			instance.instance
			.enumerate_physical_devices()
			.expect("failed to retrive physical devices");
		let surface_loader = khr::Surface::new(
			&instance.entry,
			&instance.instance,
		);
		let (physical_device, queue_family_index) =
			pdevices.iter().find_map(|pdevice| {
				instance
					.instance.get_physical_device_queue_family_properties(*pdevice)
					.iter().enumerate().find_map(|(index, info)| {
						let supports_graphic_and_surface =
							info.queue_flags.contains(vk::QueueFlags::GRAPHICS) &&
							surface_loader.get_physical_device_surface_support(
								*pdevice,
								index as u32,
								surface.surface,
							).unwrap();
						if supports_graphic_and_surface {
							Some((*pdevice, index))
						} else {
							None
						}
					})
			}).expect("Couldn't find suitable device.");
		let physical_device_memory_properties = instance.instance.get_physical_device_memory_properties(
			physical_device,
		);
		let queue_family_index = queue_family_index as u32;
		let device_queue_info =
			vk::DeviceQueueCreateInfo::builder()
			.queue_family_index(queue_family_index)
			.queue_priorities(&PRIORITIES)
			.build();
		let device_info =
			vk::DeviceCreateInfo::builder()
			.queue_create_infos(std::slice::from_ref(&device_queue_info))
			.enabled_extension_names(&DEVICE_EXTENSIONS)
			.enabled_features(&features)
			.build();
		let device =
			instance.instance
			.create_device(
				physical_device,
				&device_info,
				None
			).unwrap();
		let surface_format = surface_loader.get_physical_device_surface_formats(
			physical_device,
			surface.surface,
		).unwrap()[0];
		Self {
			device,
			queue_family_index,
			surface_loader,
			physical_device,
			physical_device_memory_properties,
			surface_format,
		}
	}}

	pub fn find_memory_type_index(
		memory_requirement: &vk::MemoryRequirements,
		memory_properties: &vk::PhysicalDeviceMemoryProperties,
		flags: vk::MemoryPropertyFlags,
	) -> u32 {
		memory_properties.memory_types[..memory_properties.memory_type_count as _]
			.iter().enumerate().find(
				|(index, memory_type)| {
					(1 << index) & memory_requirement.memory_type_bits != 0 &&
					memory_type.property_flags & flags == flags
				}
			).map(
				|(index, _)| {
					index as _
				}
			).expect("failed to find memory index")
	}
}