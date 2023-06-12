use ash::vk;

use crate::{Swapchain, Device, CommandBuffer};

pub struct RenderPass {
	pub render_pass: vk::RenderPass,
}

impl RenderPass {
	pub fn new(
		device: &Device,
		swapchain: &Swapchain,
	) -> Self { unsafe {
		let attachments = [
			vk::AttachmentDescription::builder()
				.format(swapchain.surface_format.format)
				.samples(vk::SampleCountFlags::TYPE_1)
				.load_op(vk::AttachmentLoadOp::CLEAR)
				.store_op(vk::AttachmentStoreOp::STORE)
				.final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
				.build(),
			vk::AttachmentDescription::builder()
				.format(vk::Format::D16_UNORM)
				.samples(vk::SampleCountFlags::TYPE_1)
				.load_op(vk::AttachmentLoadOp::CLEAR)
				.initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
				.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
				.build(),
		];
		let color_attachment_references = [
			vk::AttachmentReference::builder()
				.attachment(0)
				.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.build(),
		];
		let depth_attachment_reference = vk::AttachmentReference::builder()
			.attachment(1)
			.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
			.build();
		let dependencies = [
			vk::SubpassDependency::builder()
				.src_subpass(vk::SUBPASS_EXTERNAL)
				.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
				.dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ)
				.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
				.build(),
		];
		let subpass = vk::SubpassDescription::builder()
			.color_attachments(color_attachment_references.as_slice())
			.depth_stencil_attachment(&depth_attachment_reference)
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.build();
		let renderpass_info = vk::RenderPassCreateInfo::builder()
			.attachments(attachments.as_slice())
			.subpasses(std::slice::from_ref(&subpass))
			.dependencies(dependencies.as_slice())
			.build();
		let renderpass = device.device.create_render_pass(
			&renderpass_info,
			None,
		).unwrap();
		Self {
			render_pass: renderpass,
		}
	}}

	pub fn open(
		&self,
		device: &Device,
		extent: &vk::Extent2D,
		framebuffer: &vk::Framebuffer,
		command_buffer: &vk::CommandBuffer,
	) { unsafe {
		let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.render_pass(self.render_pass)
			.framebuffer(*framebuffer)
			.render_area((*extent).into())
			.clear_values(
				&[
					vk::ClearValue {
						color: vk::ClearColorValue {
							float32: [0.0, 0.0, 0.0, 1.0],
						}
					},
					vk::ClearValue {
						depth_stencil: vk::ClearDepthStencilValue {
							depth: 1.0,
							stencil: 0,
						}
					},
				]
			)
			.build();
		device.device.cmd_begin_render_pass(
			*command_buffer,
			&render_pass_begin_info,
			vk::SubpassContents::INLINE,
		);
	}}

	pub fn close(
		&self,
		device: &Device,
		command_buffer: &CommandBuffer,
	) { unsafe {
		device.device.cmd_end_render_pass(
			command_buffer.command_buffer,
		);
	}}
}