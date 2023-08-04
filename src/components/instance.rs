use std::{ffi::{CStr, c_char}, borrow::Cow, marker::PhantomData};

use ash::{vk::{self, DebugUtilsMessageSeverityFlagsEXT, InstanceCreateFlags}, extensions::{ext::DebugUtils, khr::Synchronization2}};
use raw_window_handle::HasRawDisplayHandle;

use crate::Window;

pub struct Instance {
	pub entry: ash::Entry,
	pub instance: ash::Instance,
	pub debug_callback: vk::DebugUtilsMessengerEXT,
}

impl Instance {
	pub fn new(
		name: &str,
		engine_name: &str,
		window: &Window,
	) -> Self { unsafe {
		let layer_names = [
			CStr::from_bytes_with_nul_unchecked(
				b"VK_LAYER_KHRONOS_validation\0",
			)
		];
		let layer_names_raw: Vec<*const c_char> = layer_names
			.iter().map(
				|raw_name|
				raw_name.as_ptr()
			).collect();
		let required_extensions = window.glfw.get_required_instance_extensions().unwrap_or(vec![]);
		assert!(required_extensions.contains(&"VK_KHR_surface".to_string()));
		let extensions: Vec<std::ffi::CString> = required_extensions
			.iter()
			.map(|ext| std::ffi::CString::new(ext.clone()).expect("Failed to convert extension name"))
			.collect();
		let mut extension_pointers: Vec<*const i8> = extensions.iter().map(|ext| ext.as_ptr()).collect();
		extension_pointers.push(DebugUtils::name().as_ptr());
		let name_cstr = CStr::from_bytes_with_nul_unchecked(name.as_bytes());
		let engine_name_cstr = CStr::from_bytes_with_nul_unchecked(engine_name.as_bytes());
		let entry = ash::Entry::linked();
		let application_info = vk::ApplicationInfo {
			p_application_name: name_cstr.as_ptr(),
			p_engine_name: engine_name_cstr.as_ptr(),
			api_version: vk::make_api_version(0, 1, 3, 0),
			..Default::default()
		};
		let instance_info = vk::InstanceCreateInfo::builder()
			.application_info(&application_info)
			.enabled_layer_names(&layer_names_raw)
			.enabled_extension_names(&extension_pointers)
			.build();
		let instance = entry.create_instance(
			&instance_info,
			None
		).expect("failed to construct instance");
		let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::ERROR |
				vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
				vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
			)
			.message_type(
				vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
				vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION |
				vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
			)
			.pfn_user_callback(Some(debug_callback))
			.build();
		let debug_utils = DebugUtils::new(&entry, &instance);
		let debug_callback = debug_utils
			.create_debug_utils_messenger(
				&debug_info,
				None,
		).unwrap();
		Self {
			entry,
			instance,
			debug_callback,
		}
	}}
}

unsafe extern "system" fn debug_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
	let callback_data = *p_callback_data;
	let message_id_number = callback_data.message_id_number;

	let message_id_name = if callback_data.p_message_id_name.is_null() {
		 Cow::from("")
	} else {
		 CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
	};

	let message = if callback_data.p_message.is_null() {
		 Cow::from("")
	} else {
		 CStr::from_ptr(callback_data.p_message).to_string_lossy()
	};

	println!(
		 "{message_severity:?}:\n{message_type:?} [{message_id_name} ({message_id_number})] : {message}\n",
	);

	match message_severity {
		DebugUtilsMessageSeverityFlagsEXT::ERROR => {
			println!("E");
		},
		_ => {}
	}

	vk::FALSE
}