use std::mem::align_of;

use ash::{vk, util::Align};

use crate::{Device, Instance};

pub struct BufferGOMemory(pub vk::Buffer, pub vk::DeviceMemory);

pub struct BufferGO {
	pub memory: Option<BufferGOMemory>,
	pub count: usize,
	pub capacity: usize,
	usage: vk::BufferUsageFlags,
}

impl BufferGO {
	pub fn new<T>(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
		initial_count: usize,
	) -> Self
	where T: Default + Copy + Clone { unsafe {
		let buffer_object: BufferGO;
		if initial_count == 0 {
			buffer_object = Self {
				memory: None,
				count: 0,
				capacity: initial_count,
				usage,
			};
		} else {
			let (
				buffer,
				memory,
			) = BufferGO::allocate_buffer::<T>(instance, device, usage, initial_count);
			buffer_object = Self {
				memory: Some(BufferGOMemory(buffer, memory)),
				count: 0,
				capacity: initial_count,
				usage,
			};
		}
		buffer_object
	}}

	pub fn update<T>(
		&mut self,
		instance: &Instance,
		device: &Device,
		data: &[T],
	) where T: Default + Copy + Clone { unsafe {
		if self.capacity < data.len() {
			let mut n_capacity: usize = self.capacity.max(data.len());
			while n_capacity < data.len() {
				n_capacity *= 2;
			}
			if let Some(memory) = &self.memory {
				device.device.destroy_buffer(memory.0, None);
				device.device.free_memory(memory.1, None);
			}
			let (
				buffer,
				memory,
			) = BufferGO::allocate_buffer::<T>(instance, device, self.usage, n_capacity);
			self.memory = Some(BufferGOMemory(buffer, memory));
			self.count = data.len();
			self.capacity = n_capacity;
		}
		if data.is_empty() {
			self.count = 0;
			return;
		}
		self.count = data.len();
		let memory = self.memory.as_ref().unwrap_unchecked();
		let buffer_memory_requirement = device.device.get_buffer_memory_requirements(
			memory.0,
		);
		let buffer_ptr = device.device.map_memory(
			memory.1,
			0,
			buffer_memory_requirement.size,
			vk::MemoryMapFlags::empty(),
		).unwrap();
		let mut buffer_slice = Align::new(
			buffer_ptr,
			align_of::<T>() as u64,
			buffer_memory_requirement.size,
		);
		buffer_slice.copy_from_slice(data);
		device.device.unmap_memory(memory.1);
	}}

	fn allocate_buffer<T>(
		instance: &Instance,
		device: &Device,
		usage: vk::BufferUsageFlags,
		count: usize,
	) -> (vk::Buffer, vk::DeviceMemory)
	where T: Default + Copy + Clone { unsafe {
		let buffer_info = vk::BufferCreateInfo::builder()
			.size(count as u64 * std::mem::size_of::<T>() as u64)
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
		device.device.bind_buffer_memory(
			buffer,
			memory,
			0,
		).unwrap();
		(buffer, memory)
	}}
}