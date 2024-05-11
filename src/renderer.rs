pub mod buffer;
pub mod command_buffer;
pub mod debug;
pub mod image;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod swapchain;
pub mod texture;
pub mod utility;
pub mod vkcontext;

use std::{mem::ManuallyDrop, slice};
use ash::{vk, Device};

use swapchain::Swapchain;
use vkcontext::VkContext;
use command_buffer::CommandBuffer;

const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct Renderer<'a> {
    command_buffers: ManuallyDrop<Vec<CommandBuffer<'a>>>,

    current_image_index: u32,
    current_frame: u32,

    sync_objects: Vec<SyncObject>,
    command_pool: vk::CommandPool,
    swapchain: Swapchain<'a>,
    vkcontext: &'a VkContext,
}

impl<'a> Renderer<'a> {
    pub fn new(vkcontext: &'a VkContext) -> Self {
        // Create context.

        let swapchain = Swapchain::new(&vkcontext, vkcontext.queue_family_indices, true);

        // Command pool.
        let command_pool = {
            let create_info = vk::CommandPoolCreateInfo::default()
                .queue_family_index(vkcontext.queue_family_indices.graphics_index);

            unsafe { vkcontext.device.create_command_pool(&create_info, None).unwrap() }
        };

        // Sync objects.
        let sync_objects = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                let image_available_semaphore = {
                    let create_info = vk::SemaphoreCreateInfo::default();
                    unsafe { vkcontext.device.create_semaphore(&create_info, None).unwrap() }
                };

                let queue_complete_semaphore = {
                    let create_info = vk::SemaphoreCreateInfo::default();
                    unsafe { vkcontext.device.create_semaphore(&create_info, None).unwrap() }
                };

                let in_flight_fence = {
                    let create_info = vk::FenceCreateInfo::default()
                        .flags(vk::FenceCreateFlags::SIGNALED);
                    unsafe { vkcontext.device.create_fence(&create_info, None).unwrap() }
                };

                SyncObject {
                    image_available_semaphore,
                    queue_complete_semaphore,
                    in_flight_fence,
                }
            })
            .collect::<Vec<_>>();

        let command_buffers = (0..swapchain.images.len()).map(|_| {
            CommandBuffer::new(&vkcontext, command_pool, true)
        }).collect::<Vec<_>>();

        Self {
            command_buffers: ManuallyDrop::new(command_buffers),
            current_image_index: 0,
            current_frame: 0,
            sync_objects,
            command_pool,
            swapchain,
            vkcontext,
        }
    }
}

impl<'a> Renderer<'a> {
    pub fn begin_frame(&mut self) -> bool {
        let sync_object = self.next_sync_object();

        let wait_fences = [sync_object.in_flight_fence];

        // Wait for current frame to finish rendering.
        unsafe {
            self.vkcontext.device.wait_for_fences(&wait_fences, true, std::u64::MAX).unwrap();
        }

        self.current_image_index =
            match self.swapchain.acquire_next_image_index(sync_object.image_available_semaphore) {
                Some(next_index) => next_index,
                None => return true,
        };

        unsafe { self.vkcontext.device.reset_fences(&wait_fences).unwrap() };

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

        // TEMP
        // Transition to trace.
        command_buffer.transition_image(
            self.swapchain.images[self.current_image_index as usize],
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::AccessFlags::NONE_KHR,
            vk::AccessFlags::MEMORY_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        unsafe {
            self.vkcontext.device.cmd_dispatch(
                command_buffer.handle,
                (self.swapchain.swapchain_properties.extent.width as f32 / 8.0f32).ceil() as u32,
                (self.swapchain.swapchain_properties.extent.height as f32 / 8.0f32).ceil() as u32,
                1,
            );
        }

        false
    }

    pub fn end_frame(&mut self) -> bool {
        let sync_object = self.current_sync_object();

        let command_buffer = &self.command_buffers[self.current_frame as usize];

        let in_flight_fence = sync_object.in_flight_fence;
        let wait_semaphores = [sync_object.image_available_semaphore];
        let signal_semaphores = [sync_object.queue_complete_semaphore];

        // Transition to present.
        command_buffer.transition_image(
            self.swapchain.images[self.current_image_index as usize],
            vk::PipelineStageFlags::COMPUTE_SHADER,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::AccessFlags::MEMORY_WRITE,
            vk::AccessFlags::NONE_KHR,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        command_buffer.end(&self.vkcontext);

        // Submit queue.
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(slice::from_ref(&vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT))
            .command_buffers(slice::from_ref(&command_buffer.handle))
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.vkcontext.device.queue_submit(
                self.vkcontext.graphics_queue,
                slice::from_ref(&submit_info),
                in_flight_fence
            ).unwrap();
        }

        // Present.
        if self.swapchain.present(
            sync_object.image_available_semaphore,
            self.current_image_index
        ) {
            return true;
        }

        false
    }
}

impl<'a> Renderer<'a> {
    fn next_sync_object(&mut self) -> SyncObject {
        let next = self.sync_objects[self.current_frame as usize];

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;

        next
    }

    fn current_sync_object(&self) -> SyncObject {
        self.sync_objects[self.current_frame as usize]
    }

    pub fn recreate_swapchain(&mut self) {
        log::debug!("Recreating swapchain.");

        self.vkcontext.wait_gpu_idle();

        let swapchain = Swapchain::new(&self.vkcontext, self.vkcontext.queue_family_indices, true);

        self.swapchain = swapchain;
    }
}

impl<'a> Drop for Renderer<'a> {
    fn drop(&mut self) {
        log::debug!("Dropping renderer.");

        let device = &self.vkcontext.device;

        unsafe {
            for sync_object in self.sync_objects.iter() {
                sync_object.destroy(device);
            }

            ManuallyDrop::drop(&mut self.command_buffers);

            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

#[derive(Clone, Copy)]
struct SyncObject {
    image_available_semaphore: vk::Semaphore,
    queue_complete_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

impl SyncObject {
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_semaphore(self.image_available_semaphore, None);
            device.destroy_semaphore(self.queue_complete_semaphore, None);
            device.destroy_fence(self.in_flight_fence, None);
        }
    }
}
