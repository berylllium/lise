use ash::khr::surface;
use ash::{vk, Instance};
use crate::math::vec2::Vec2;

use super::image::Image;
use super::vkcontext::{VkContext, QueueFamilyIndices};
use super::utility::create_image_view;

pub struct Swapchain<'ctx> {
    pub out_of_date: bool,

    pub image_views: Vec<vk::ImageView>,
    pub images: Vec<vk::Image>,
    pub depth_images: Option<Vec<Image<'ctx>>>,

    pub swapchain_properties: SwapchainProperties,

    pub handle: vk::SwapchainKHR,

    vkcontext: &'ctx VkContext,
}

impl<'ctx> Swapchain<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        queue_family_indices: QueueFamilyIndices,
        create_depth_attachments: bool,
    ) -> Self {
        let details = SwapchainSupportDetails::query(
            &vkcontext.instance,
            vkcontext.physical_device,
            &vkcontext.loaders.surface_instance,
            vkcontext.surface_khr,
        );

        let properties = details.get_ideal_swapchain_properties();

        let format = properties.format;
        let present_mode = properties.present_mode;
        let extent = properties.extent;

        let image_count = {
            let max = details.capabilities.max_image_count;
            let mut preferred = details.capabilities.min_image_count + 1;
            if max > 0 && preferred > max {
                preferred = max;
            }
            preferred
        };

        log::debug!(
            "Creating swapchain.\n\tFormat: {:?}\n\tColorSpace:{:?}\n\tPresentMode:{:?}\n\tExtent:{:?}\n\tImageCount:{:?}",
            format.format,
            format.color_space,
            present_mode,
            extent,
            image_count,
        );

        let graphics = queue_family_indices.graphics_index;
        let present = queue_family_indices.present_index;
        let families_indices = [graphics, present];

        let create_info = {
            let mut create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(vkcontext.surface_khr)
                .min_image_count(image_count)
                .image_format(format.format)
                .image_color_space(format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::STORAGE);

            create_info = if graphics != present {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&families_indices)
            } else {
                create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            };

            create_info
                .pre_transform(details.capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
        };

        let swapchain =
            unsafe { vkcontext.loaders.swapchain_device.create_swapchain(&create_info, None).unwrap() };
        let images = unsafe { vkcontext.loaders.swapchain_device.get_swapchain_images(swapchain).unwrap() };
        
        let image_views = images
            .iter()
            .map(|image| {
                create_image_view(
                    &vkcontext.device,
                    *image,
                    properties.format.format,
                    vk::ImageAspectFlags::COLOR,
                    1
                )
            })
            .collect::<Vec<_>>();
        
        let depth_images = if create_depth_attachments {
            details.depth_format.map(|depth_format| {
                (0..images.len()).map(|_| {
                    Image::new(
                        vkcontext,
                        vk::ImageType::TYPE_2D,
                        Vec2::new(properties.extent.width, properties.extent.height),
                        depth_format,
                        vk::ImageTiling::OPTIMAL,
                        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                        vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        Some(vk::ImageAspectFlags::DEPTH),
                    )
                })
                .collect::<Vec<_>>()
            })
        } else {
            None
        };

        Self {
            out_of_date: false,
            image_views,
            images,
            swapchain_properties: properties,
            handle: swapchain,
            depth_images,
            vkcontext,
        }
    }
}

impl<'ctx> Swapchain<'ctx> {
    pub fn acquire_next_image_index(&mut self, image_available_semaphore: vk::Semaphore) -> Option<u32> {
        let result = unsafe {
            self.vkcontext.loaders.swapchain_device.acquire_next_image(
                self.handle,
                std::u64::MAX,
                image_available_semaphore,
                vk::Fence::null())
        };

        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Swapchain out of date.");
                self.out_of_date = true;
                return None;
            },
            _ => return None
        };

        Some(image_index)
    }

    pub fn present(&mut self, render_complete_semaphore: vk::Semaphore, present_image_index: u32) -> bool {
        let wait_semaphores = [render_complete_semaphore];
        let swapchains = [self.handle];
        let image_indices = [present_image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let result = unsafe {
            self.vkcontext.loaders.swapchain_device.queue_present(self.vkcontext.present_queue, &present_info)
        };

        match result {
            Ok(true) => return true,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                log::debug!("Swapchain out of date.");
                self.out_of_date = true;
            },
            Err(error) => panic!("Failed to present swapchain: {}", error),
            _ => {}
        }

        false
    }
}

impl<'ctx> Drop for Swapchain<'ctx> {
    fn drop(&mut self) {
        // Free image views.
        for image_view in self.image_views.iter() {
            unsafe { self.vkcontext.device.destroy_image_view(*image_view, None) };
        }

        unsafe { self.vkcontext.loaders.swapchain_device.destroy_swapchain(self.handle, None) };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SwapchainProperties {
    pub format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
    pub depth_format: Option<vk::Format>,
}

impl SwapchainSupportDetails {
    pub const DEPTH_ATTACHMENT_FORMAT_CANDIDATES: [vk::Format; 3] = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    pub fn query(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_instance_loader: &surface::Instance,
        surface: vk::SurfaceKHR
    ) -> Self {
        let capabilities = unsafe {
            surface_instance_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };

        let formats = unsafe {
            surface_instance_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()
        };

        let present_modes = unsafe {
            surface_instance_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };

        let depth_format = Self::DEPTH_ATTACHMENT_FORMAT_CANDIDATES.iter().cloned().find(|&format| {
            let flag = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;
            let format_properties = unsafe {
                instance.get_physical_device_format_properties(physical_device, format)
            };

            (format_properties.linear_tiling_features & flag) == flag
            || (format_properties.optimal_tiling_features & flag) == flag
        });

        Self {
            capabilities,
            formats,
            present_modes,
            depth_format,
        }
    }

    pub fn get_ideal_swapchain_properties(&self) -> SwapchainProperties {
        let format = Self::choose_swapchain_surface_format(&self.formats);
        let present_mode = Self::choose_swapchain_surface_present_mode(&self.present_modes);
        let extent = Self::choose_swapchain_extent(self.capabilities);

        SwapchainProperties {
            format,
            present_mode,
            extent,
        }
    }

    fn choose_swapchain_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        if available_formats.len() == 1 && available_formats[0].format == vk::Format::UNDEFINED {
            return vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            };
        }

        *available_formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&available_formats[0])
    }

    fn choose_swapchain_surface_present_mode(available_present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if available_present_modes.contains(&vk::PresentModeKHR::FIFO) {
            vk::PresentModeKHR::FIFO
        } else {
            vk::PresentModeKHR::IMMEDIATE
        }
    }

    fn choose_swapchain_extent(capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        capabilities.current_extent
    }

}
