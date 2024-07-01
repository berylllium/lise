pub mod buffer;
pub mod command_buffer;
pub mod debug;
pub mod frame_buffer;
pub mod image;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod swapchain;
pub mod texture;
pub mod utility;
pub mod vkcontext;

use std::{mem::ManuallyDrop, slice};
use ash::vk;

use swapchain::Swapchain;
use vkcontext::VkContext;
use command_buffer::CommandBuffer;

use crate::math::vec2::Vec2UI;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct Renderer<'ctx> {
    pub command_buffers: ManuallyDrop<Vec<CommandBuffer<'ctx>>>,

    pub current_image_index: u32,
    pub current_frame: u32,

    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub queue_complete_semaphores: Vec<vk::Semaphore>,
    pub queue_complete_fences: Vec<vk::Fence>,
    pub queue_complete_fences_image: Vec<Option<vk::Fence>>,
    
    pub command_pool: vk::CommandPool,
    pub swapchain: Swapchain<'ctx>,
    vkcontext: &'ctx VkContext,
}

impl<'ctx> Renderer<'ctx> {
    pub fn new(vkcontext: &'ctx VkContext) -> Self {
        let swapchain = Swapchain::new(&vkcontext, vkcontext.queue_family_indices, true);

        // Command pool.
        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(vkcontext.queue_family_indices.graphics_index);

            unsafe { vkcontext.device.create_command_pool(&create_info, None).unwrap() }
        };

        // Sync objects.
        let image_available_semaphores = (0..MAX_FRAMES_IN_FLIGHT).map(|_| {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { vkcontext.device.create_semaphore(&create_info, None).unwrap() }
        }).collect::<Vec<_>>();

        let queue_complete_semaphores = (0..MAX_FRAMES_IN_FLIGHT).map(|_| {
            let create_info = vk::SemaphoreCreateInfo::default();
            unsafe { vkcontext.device.create_semaphore(&create_info, None).unwrap() }
        }).collect::<Vec<_>>();

        let queue_complete_fences = (0..MAX_FRAMES_IN_FLIGHT).map(|_| {
            let create_info = vk::FenceCreateInfo::default()
                .flags(vk::FenceCreateFlags::SIGNALED);
            unsafe { vkcontext.device.create_fence(&create_info, None).unwrap() }
        }).collect::<Vec<_>>();

        let command_buffers = swapchain.images.iter().map(|_| {
            CommandBuffer::new(&vkcontext, command_pool, true)
        }).collect::<Vec<_>>();

        Self {
            command_buffers: ManuallyDrop::new(command_buffers),
            current_image_index: 0,
            current_frame: 0,
            image_available_semaphores,
            queue_complete_semaphores,
            queue_complete_fences,
            queue_complete_fences_image: vec![None; swapchain.images.len()],
            command_pool,
            swapchain,
            vkcontext,
        }
    }
}

impl<'ctx> Renderer<'ctx> {
    pub fn prepare_frame(&mut self) -> bool {
        if self.swapchain.out_of_date {
            self.recreate_swapchain()
        }

        // Wait for current frame to finish rendering.
        unsafe {
            self.vkcontext.device.wait_for_fences(
                slice::from_ref(&self.queue_complete_fences[self.current_frame as usize]),
                true,
                u64::MAX
            ).unwrap();
        }

        // Get next swapchain image index.
        self.current_image_index = match self.swapchain.acquire_next_image_index(self.image_available_semaphores[self.current_frame as usize]) {
            Some(next_index) => next_index,
            None => return true,
        };

        // Begin command buffer.
        let command_buffer = &self.command_buffers[self.current_frame as usize];
        command_buffer.begin(false, false, false);

        // Dynamic State.
        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(100.0)
            .width(self.swapchain.swapchain_properties.extent.width as f32)
            .height(self.swapchain.swapchain_properties.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D::default().x(0).y(0))
            .extent(
                vk::Extent2D::default()
                    .width(self.swapchain.swapchain_properties.extent.width)
                    .height(self.swapchain.swapchain_properties.extent.height)
            );

        unsafe {
            self.vkcontext.device.cmd_set_viewport(command_buffer.handle, 0, slice::from_ref(&viewport));
            self.vkcontext.device.cmd_set_scissor(command_buffer.handle, 0, slice::from_ref(&scissor));
        }

        false
    }

    pub fn submit_frame(&mut self) -> bool {
        let command_buffer = &self.command_buffers[self.current_frame as usize];

        command_buffer.end(&self.vkcontext);

        // Wait if a previous frame is still using this image.
        match self.queue_complete_fences_image[self.current_image_index as usize] {
            Some(fence) => unsafe {
                self.vkcontext.device.wait_for_fences(
                    slice::from_ref(&fence),
                    true,
                    u64::MAX
                ).unwrap();
            },
            None => (),
        }

        // Mark fence as  being in use by this image.
        self.queue_complete_fences_image[self.current_image_index as usize] =
            Some(self.queue_complete_fences[self.current_frame as usize]);

        // Reset fence.
        unsafe {
            self.vkcontext.device.reset_fences(slice::from_ref(&self.queue_complete_fences_image[self.current_image_index as usize].unwrap())).unwrap();
        }

        // Submit queue.
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(slice::from_ref(&self.image_available_semaphores[self.current_frame as usize]))
            .wait_dst_stage_mask(slice::from_ref(&vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT))
            .command_buffers(slice::from_ref(&command_buffer.handle))
            .signal_semaphores(slice::from_ref(&self.queue_complete_semaphores[self.current_frame as usize]));

        unsafe {
            self.vkcontext.device.queue_submit(
                self.vkcontext.graphics_queue,
                slice::from_ref(&submit_info),
                self.queue_complete_fences[self.current_frame as usize]
            ).unwrap();
        }

        // Present.
        if self.swapchain.present(
            self.queue_complete_semaphores[self.current_frame as usize],
            self.current_image_index
        ) {
            return true;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        false
    }

    pub fn get_current_command_buffer_handle(&self) -> vk::CommandBuffer {
        self.command_buffers[self.current_frame as usize].handle
    }

    pub fn get_render_area_size(&self) -> Vec2UI {
        Vec2UI::from_vk_extent_2d(self.swapchain.swapchain_properties.extent)
    }
}

impl<'ctx> Renderer<'ctx> {
    pub fn recreate_swapchain(&mut self) {
        log::debug!("Recreating swapchain.");

        self.vkcontext.wait_gpu_idle();

        let swapchain = Swapchain::new(&self.vkcontext, self.vkcontext.queue_family_indices, true);

        self.swapchain = swapchain;
    }
}

impl<'ctx> Drop for Renderer<'ctx> {
    fn drop(&mut self) {
        log::debug!("Dropping renderer.");

        let device = &self.vkcontext.device;

        unsafe {
            for sem in self.image_available_semaphores.iter() {
                device.destroy_semaphore(*sem, None);
            }

            for sem in self.queue_complete_semaphores.iter() {
                device.destroy_semaphore(*sem, None);
            }

            for fence in self.queue_complete_fences.iter() {
                device.destroy_fence(*fence, None);
            }

            ManuallyDrop::drop(&mut self.command_buffers);

            device.destroy_command_pool(self.command_pool, None);
        }
    }
}
