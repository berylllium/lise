use ash::vk;

use crate::math::vec2::Vec2UI;

use super::vkcontext::VkContext;

pub struct Framebuffer<'ctx> {
    pub handle: vk::Framebuffer,

    vkcontext: &'ctx VkContext,
}

impl<'ctx> Framebuffer<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        render_pass: vk::RenderPass,
        attachments: &[vk::ImageView],
        render_area_size: Vec2UI,
    ) -> Self {
        let create_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(render_area_size.x)
            .height(render_area_size.y)
            .layers(1);

        let handle = unsafe { vkcontext.device.create_framebuffer(&create_info, None).unwrap() };

        Self {
            handle,
            vkcontext,
        }
    }
}

impl<'ctx> Drop for Framebuffer<'ctx> {
    fn drop(&mut self) {
        unsafe {
            self.vkcontext.device.destroy_framebuffer(self.handle, None);
        }
    }
}
