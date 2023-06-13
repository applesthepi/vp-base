use std::mem::align_of;

use ash::{vk, util::Align};

use crate::{Device, Instance};

pub struct Buffer<T> {
	pub data: Vec<T>,
	pub buffer: vk::Buffer,
	pub memory: vk::DeviceMemory,
}

impl<T> Buffer<T>
where
	T: Default + Copy + Clone {
	pub fn with_default(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
	) -> Self {
		let mut data = Vec::with_capacity(1);
		data.resize(1, T::default());
		Buffer::new(
			instance,
			device,
			usage,
			data,
		)
	}

	pub fn with_size(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
		size: usize,
	) -> Self {
		let mut data = Vec::with_capacity(size);
		data.resize(size, T::default());
		Buffer::new(
			instance,
			device,
			usage,
			data,
		)
	}

	pub fn with_data(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
		data: &[T],
	) -> Self {
		let mut v_data = Vec::with_capacity(data.len());
		v_data.extend_from_slice(data);
		let buffer = Buffer::new(
			instance,
			device,
			usage,
			v_data,
		);
		// TODO: set data
		buffer
	}

	fn new(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
		data: Vec<T>,
	) -> Self { unsafe {
		let buffer_info = vk::BufferCreateInfo::builder()
			.size(data.len() as u64)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE)
			.build();
		let buffer = device.device.create_buffer(
			&buffer_info,
			None,
		).unwrap();
		let buffer_memory_requirement = device.device.get_buffer_memory_requirements(
			buffer,
		);
		let device_memory_properties = instance.instance.get_physical_device_memory_properties(
			device.physical_device
		);
		let buffer_memory_index = Device::find_memory_type_index(
			&buffer_memory_requirement,
			&device_memory_properties,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
		);
		let allocate_info = vk::MemoryAllocateInfo::builder()
			.allocation_size(buffer_memory_requirement.size)
			.memory_type_index(buffer_memory_index)
			.build();
		let memory = device.device.allocate_memory(
			&allocate_info,
			None,
		).unwrap();
		let buffer_ptr = device.device.map_memory(
			memory,
			0,
			buffer_memory_requirement.size,
			vk::MemoryMapFlags::empty(),
		).unwrap();
		let mut buffer_slice = Align::new(
			buffer_ptr,
			align_of::<u32>() as u64,
			buffer_memory_requirement.size,
		);
		buffer_slice.copy_from_slice(data.as_slice());
		device.device.unmap_memory(memory);
		device.device.bind_buffer_memory(
			buffer,
			memory,
			0,
		).unwrap();
		Self {
			data,
			buffer,
			memory,
		}
	}}
}