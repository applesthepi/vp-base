use ash::vk;

use crate::Device;

pub struct Fence {
	pub fence: vk::Fence,
}

impl Fence {
	pub fn new(
		device: &Device,
	) -> Self { unsafe {
		let fence_info = vk::FenceCreateInfo::builder()
			.flags(vk::FenceCreateFlags::SIGNALED)
			.build();
		let fence = device.device.create_fence(&fence_info, None).unwrap();
		Self {
			fence,
		}
	}}
}