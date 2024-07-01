use core::slice;

use ash::vk;
use super::vkcontext::VkContext;

pub struct CommandBuffer<'ctx> {
    pub handle: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    vkcontext: &'ctx VkContext,
}

impl<'ctx> CommandBuffer<'ctx> {
    pub fn new(vkcontext: &'ctx VkContext, command_pool: vk::CommandPool, is_primary: bool) -> Self {
        let handle = {
            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(if is_primary {vk::CommandBufferLevel::PRIMARY} else {vk::CommandBufferLevel::SECONDARY})
                .command_buffer_count(1);

            unsafe { vkcontext.device.allocate_command_buffers(&allocate_info).ok().unwrap()[0] }
        };

        Self {
            handle,
            command_pool,
            vkcontext,
        }
    }
}

impl<'ctx> CommandBuffer<'ctx> {
    pub fn begin(&self, is_single_use: bool, is_render_pass_continue: bool, is_simultaneous_use: bool) {
        let mut flags = vk::CommandBufferUsageFlags::default();

        if is_single_use { flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT; }
        if is_render_pass_continue { flags |= vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE; }
        if is_simultaneous_use { flags |= vk::CommandBufferUsageFlags::SIMULTANEOUS_USE; }

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(flags);

        unsafe { self.vkcontext.device.begin_command_buffer(self.handle, &begin_info).unwrap() }
    }

    pub fn end(&self, vkcontext: &VkContext) {
        unsafe { vkcontext.device.end_command_buffer(self.handle).unwrap() }
    }

    pub fn end_and_submit_single_use(&self, queue: vk::Queue) {
        let buffers = [self.handle];

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&buffers);

        unsafe {
            self.vkcontext.device.queue_submit(
                queue,
                std::slice::from_ref(&submit_info),
                vk::Fence::null()
            )
            .unwrap()
        }
    }
}

impl<'ctx> CommandBuffer<'ctx> {
    pub fn transition_image(
        &self,
        image: vk::Image,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        src_access: vk::AccessFlags,
        dst_access: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let resource_range = vk::ImageSubresourceRange::default()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = vk::ImageMemoryBarrier::default()
            .src_access_mask(src_access)
            .dst_access_mask(dst_access)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(resource_range);

        unsafe {
            self.vkcontext.device.cmd_pipeline_barrier(
                self.handle,
                src_stage,
                dst_stage,
                vk::DependencyFlags::default(),
                &[],
                &[],
                slice::from_ref(&barrier),
            );
        }
    }
}

impl<'ctx> Drop for CommandBuffer<'ctx> {
    fn drop(&mut self) {
        unsafe { self.vkcontext.device.free_command_buffers(self.command_pool, &[self.handle]) }
    }
}
