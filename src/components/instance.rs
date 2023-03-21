use std::ffi::CStr;

use ash::vk;

pub struct Instance {
	pub entry: ash::Entry,
	pub instance: ash::Instance,
}

impl Instance {
	pub fn new(
		name: &str,
		engine_name: &str,
	) -> Self { unsafe {
		let name_cstr = CStr::from_bytes_with_nul_unchecked(name.as_bytes());
		let engine_name_cstr = CStr::from_bytes_with_nul_unchecked(engine_name.as_bytes());
		let entry = ash::Entry::linked();
		let application_info = vk::ApplicationInfo {
			p_application_name: name_cstr.as_ptr(),
			p_engine_name: engine_name_cstr.as_ptr(),
			api_version: vk::make_api_version(0, 1, 3, 0),
			..Default::default()
		};
		let instance_info = vk::InstanceCreateInfo {
			p_application_info: &application_info,
			..Default::default()
		};
		let instance = entry.create_instance(
			&instance_info,
			None
		).expect("failed to construct instance");
		Self {
			entry,
			instance,
		}
	}}
}