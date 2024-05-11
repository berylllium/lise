use ash::{vk, Device};

pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let create_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        });

    unsafe { device.create_image_view(&create_info, None).unwrap() }
}

pub fn query_memory_type(
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    memory_requirements: vk::MemoryRequirements,
    memory_flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    (0..memory_properties.memory_type_count).fold(None, |acc, e| {
        if memory_requirements.memory_type_bits & (1 << e) != 0
            && (memory_properties.memory_types[e as usize].property_flags & memory_flags) == memory_flags
            && (memory_properties.memory_types[e as usize].property_flags & vk::MemoryPropertyFlags::DEVICE_COHERENT_AMD).as_raw() == 0 {
            Some(e)
        } else {
            acc
        }
    })
}
