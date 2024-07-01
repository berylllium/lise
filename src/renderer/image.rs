use ash::vk;

use crate::math::vec2::Vec2UI;

use super::{utility, vkcontext::VkContext};

pub struct Image<'ctx> {
    pub handle: vk::Image,
    pub format: vk::Format,
    pub size: Vec2UI,

    pub memory: vk::DeviceMemory,

    pub image_view: Option<vk::ImageView>,

    vkcontext: &'ctx VkContext,
}

impl<'ctx> Image<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        image_type: vk::ImageType,
        size: Vec2UI,
        format: vk::Format,
        tiling: vk::ImageTiling,
        use_flags: vk::ImageUsageFlags,
        memory_flags: vk::MemoryPropertyFlags,
        view_aspect_flags: Option<vk::ImageAspectFlags>,
    ) -> Self {
        let handle = {
            let create_info = vk::ImageCreateInfo::default()
                .image_type(image_type)
                .format(format)
                .extent(size.as_vk_extent_3d(1))
                .mip_levels(4)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(tiling)
                .usage(use_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED);

            unsafe { vkcontext.device.create_image(&create_info, None).unwrap() }
        };

        let memory_properties = vkcontext.physical_device_memory_properties;
        let memory_requirements = unsafe { vkcontext.device.get_image_memory_requirements(handle) };

        let memory_type = utility::query_memory_type(memory_properties, memory_requirements, memory_flags);

        let memory = {
            let allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type.unwrap());

            unsafe { vkcontext.device.allocate_memory(&allocate_info, None ).unwrap() }
        };

        unsafe { vkcontext.device.bind_image_memory(handle, memory, 0).unwrap() }

        let image_view = view_aspect_flags.map(|aspect_flags| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(handle)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange::default()
                    .aspect_mask(aspect_flags)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                );

            unsafe { vkcontext.device.create_image_view(&create_info, None).unwrap() }
        });

        Self {
            handle,
            format,
            size,
            memory,
            image_view,
            vkcontext,
        }
    }
}

impl<'ctx> Image<'ctx> {
    pub fn transition_layout(
        &self,
        command_buffer: vk::CommandBuffer,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
    ) {
        let barrier = vk::ImageMemoryBarrier::default()
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(self.handle)
            .subresource_range(vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
            );

        unsafe {
            self.vkcontext.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                dst_stage,
                vk::DependencyFlags::default(),
                &[],
                &[],
                std::slice::from_ref(&barrier)
            );
        }

    }

    pub fn transition_undefined_to_transfer_dst_optimal(&self, command_buffer: vk::CommandBuffer) {
        self.transition_layout(
            command_buffer, 
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags::default(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        );
    }

    pub fn transition_transfer_dst_optimal_to_shader_read_only_optimal(&self, command_buffer: vk::CommandBuffer) {
        self.transition_layout(
            command_buffer, 
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        );
    }

    pub fn copy_from_buffer(&self, command_buffer: vk::CommandBuffer, buffer: vk::Buffer) {
        let copy_info = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
            )
            .image_offset(vk::Offset3D::default())
            .image_extent(self.size.as_vk_extent_3d(1));

            unsafe { 
                self.vkcontext.device.cmd_copy_buffer_to_image(
                    command_buffer,
                    buffer,
                    self.handle,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    std::slice::from_ref(&copy_info)
                );
            }
    }
}

impl<'ctx> Drop for Image<'ctx> {
    fn drop(&mut self) {
        unsafe {
            match self.image_view {
                Some(v) => self.vkcontext.device.destroy_image_view(v, None),
                None => (),
            }

            self.vkcontext.device.free_memory(self.memory, None);

            self.vkcontext.device.destroy_image(self.handle, None);
        }
    }
}
