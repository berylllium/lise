use ash::vk;

use crate::math::vec2::Vec2UI;

use super::{command_buffer::CommandBuffer, vkcontext::VkContext};

pub struct RenderPass<'c> {
    pub handle: vk::RenderPass,
    pub render_area_start: Vec2UI,
    pub render_area_size: Vec2UI,
    pub attachment_clear_values: Vec<vk::ClearValue>,
    vkcontext: &'c VkContext,
}

impl<'c> RenderPass<'c> {
    pub fn new(
        vkcontext: &'c VkContext,
        render_area_start: Vec2UI,
        render_area_size: Vec2UI,
        attachments: &[vk::AttachmentDescription],
        attachment_clear_values: &[Option<vk::ClearValue>],
        subpasses: &[RenderPassSubPassInfo],
        dependencies: &[vk::SubpassDependency],
    ) -> Self {
        let subpasses = subpasses.iter().map(|info| {
            let mut description = vk::SubpassDescription::default()
                .pipeline_bind_point(info.bind_point)
                .input_attachments(info.input_attachments);

            info.resolve_attachments.inspect(|&s| { description = description.resolve_attachments(s); });
            info.color_attachments.inspect(|&s| { description = description.color_attachments(s); });
            info.preserve_attachments.inspect(|&s| { description = description.preserve_attachments(s); });
            info.depth_stencil_attachments.inspect(|&s| { description = description.depth_stencil_attachment(s); });

            description
        })
        .collect::<Vec<_>>();

        let handle = {
            let create_info = vk::RenderPassCreateInfo::default()
                .attachments(attachments)
                .subpasses(&subpasses)
                .dependencies(dependencies);

            unsafe { vkcontext.device.create_render_pass(&create_info, None).unwrap() }
        };

        let attachment_clear_values = attachment_clear_values.iter().map(|v| {
            match v {
                Some(clear_value) => *clear_value,
                None => vk::ClearValue::default(),
            }
        })
        .collect::<Vec<_>>();

        Self {
            handle,
            render_area_start,
            render_area_size,
            attachment_clear_values,
            vkcontext,
        }
    }
}

impl<'c> RenderPass<'c> {
    pub fn begin(&self, command_buffer: vk::CommandBuffer, frame_buffer: vk::Framebuffer) {
        let begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.handle)
            .framebuffer(frame_buffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.render_area_size.as_vk_extent_2d(),
            })
            .clear_values(&self.attachment_clear_values);

        unsafe { self.vkcontext.device.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::INLINE); }
    }

    pub fn end(&self, command_buffer: vk::CommandBuffer) {
        unsafe { self.vkcontext.device.cmd_end_render_pass(command_buffer); }
    }
}

impl<'c> Drop for RenderPass<'c> {
    fn drop(&mut self) {
        unsafe { self.vkcontext.device.destroy_render_pass(self.handle, None); }
    }
}

pub struct RenderPassSubPassInfo<'a> {
    pub bind_point: vk::PipelineBindPoint,
    pub input_attachments: &'a [vk::AttachmentReference],
    pub color_attachments: Option<&'a [vk::AttachmentReference]>,
    pub resolve_attachments: Option<&'a [vk::AttachmentReference]>,
    pub depth_stencil_attachments: Option<&'a vk::AttachmentReference>,
    pub preserve_attachments: Option<&'a [u32]>,
}
