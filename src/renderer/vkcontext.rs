use ash::{
    ext::debug_utils, khr::{surface, swapchain}, vk, Device, Entry, Instance
};
use simple_window::Window;
use std::ffi::{CStr, CString};
use super::swapchain::SwapchainSupportDetails;
use super::debug::*;

pub struct VkContext {
    pub queue_family_indices: QueueFamilyIndices,
    pub present_queue: vk::Queue,
    pub graphics_queue: vk::Queue,
    pub device: Device,
    pub physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub physical_device: vk::PhysicalDevice,
    pub surface_khr: vk::SurfaceKHR,
    pub debug_report_callback: Option<(debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    pub instance: Instance,
    pub loaders: ExtensionLoaders,
    pub entry: Entry,
}

impl VkContext {
    pub fn new(window: &Window) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load ash entry.") };
        let instance = Self::create_instance(&entry, window);

        let surface_instance_loader = surface::Instance::new(&entry, &instance);

        let surface_khr = unsafe { 
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None
            )
            .unwrap()
        };

        let debug_report_callback = setup_debug_messenger(&entry, &instance);

        let (physical_device, physical_device_properties, queue_family_indices) =
            Self::pick_physical_device(&instance, &surface_instance_loader, surface_khr);
        
        let physical_device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (device, graphics_queue, present_queue) = 
            Self::create_logical_device_with_graphics_queue(&instance, physical_device, queue_family_indices);

        let swapchain_instance_loader = swapchain::Instance::new(&entry, &instance);
        let swapchain_device_loader = swapchain::Device::new(&instance, &device);

        VkContext {
            queue_family_indices,
            present_queue,
            graphics_queue,
            device,
            physical_device_memory_properties,
            physical_device_properties,
            physical_device,
            debug_report_callback,
            surface_khr,
            instance,
            loaders: ExtensionLoaders {
                surface_instance: surface_instance_loader,
                swapchain_instance: swapchain_instance_loader,
                swapchain_device: swapchain_device_loader,
            },
            entry,
        }
    }

}

impl VkContext {
    pub fn wait_gpu_idle(&self) {
        unsafe { self.device.device_wait_idle().unwrap(); }
    }
}

impl VkContext {
    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_name = CString::new("Industria").unwrap();
        let engine_name = CString::new("No Engine").unwrap();
        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(engine_name.as_c_str())
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        let extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap();

        let mut extension_names = extension_names.iter().map(|ext| *ext).collect::<Vec<_>>();

        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(debug_utils::NAME.as_ptr());
        }

        let (_layer_names, layer_names_ptr) = get_layer_names_and_pointers();

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .flags(vk::InstanceCreateFlags::default());

        if ENABLE_VALIDATION_LAYERS {
            check_validation_layer_support(&entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_names_ptr);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn pick_physical_device(
        instance: &Instance,
        surface_loader: &surface::Instance,
        surface_khr: vk::SurfaceKHR,
    ) -> (vk::PhysicalDevice, vk::PhysicalDeviceProperties, QueueFamilyIndices) {
        let devices = unsafe { instance.enumerate_physical_devices().unwrap() };
        let device = devices
            .into_iter()
            .find(|device| Self::is_device_suitable(instance, surface_loader, surface_khr, *device))
            .expect("No suitable physical devices found.");

        let props = unsafe { instance.get_physical_device_properties(device) };
        
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        let (graphics, present) = Self::find_queue_families(instance, surface_loader, surface_khr, device);

        let queue_families_indices = QueueFamilyIndices {
            graphics_index: graphics.unwrap(),
            present_index: present.unwrap(),
        };

        (device, props, queue_families_indices)
    }

    fn is_device_suitable(
        instance: &Instance,
        surface_loader: &surface::Instance,
        surface_khr: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> bool {
        let (graphics, present) = Self::find_queue_families(instance, surface_loader, surface_khr, device);
        let extension_support = Self::check_device_extension_support(instance, device);

        let is_swapchain_suitable = {
            let details = SwapchainSupportDetails::query(instance, device, surface_loader, surface_khr);
            !details.formats.is_empty() && !details.present_modes.is_empty()
        };

        let features = unsafe { instance.get_physical_device_features(device) };

        graphics.is_some()
            && present.is_some()
            && extension_support
            && is_swapchain_suitable
            && features.sampler_anisotropy == vk::TRUE
    }

    fn check_device_extension_support(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        let required_extensions = Self::get_required_device_extensions();

        let extension_props = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .unwrap()
        };

        for required in required_extensions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }

        true
    }

    fn get_required_device_extensions() -> [&'static CStr; 1] {
        [swapchain::NAME]
    }

    fn find_queue_families(
        instance: &Instance,
        surface_loader: &surface::Instance,
        surface_khr: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> (Option<u32>, Option<u32>) {
        let mut graphics = None;
        let mut present = None;

        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, family) in props.iter().filter(|f| f.queue_count > 0).enumerate() {
            let index = index as u32;

            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics.is_none() {
                graphics = Some(index);
            }

            let present_support = unsafe {
                surface_loader.
                    get_physical_device_surface_support(device, index, surface_khr)
                    .unwrap()
            };

            if present_support && present.is_none() {
                present = Some(index);
            }

            if graphics.is_some() && present.is_some() {
                break;
            }
        }

        (graphics, present)
    }

    fn create_logical_device_with_graphics_queue(
        instance: &Instance,
        device: vk::PhysicalDevice,
        queue_family_indices: QueueFamilyIndices,
    ) -> (Device, vk::Queue, vk::Queue) {
        let graphics_family_index = queue_family_indices.graphics_index;
        let present_family_index = queue_family_indices.present_index;
        let queue_priorities = [1.0f32];

        let queue_create_infos = {
            let mut indices = vec![graphics_family_index, present_family_index];
            indices.dedup();

            indices
                .iter()
                .map(|index| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*index)
                        .queue_priorities(&queue_priorities)
                })
                .collect::<Vec<_>>()
        };

        let device_extensions = Self::get_required_device_extensions();
        let device_extensions_ptrs = device_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let device_features = vk::PhysicalDeviceFeatures::default()
            .sampler_anisotropy(true);

        let mut vk11_device_features = vk::PhysicalDeviceVulkan11Features::default()
            .storage_buffer16_bit_access(true)
            .uniform_and_storage_buffer16_bit_access(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions_ptrs)
            .enabled_features(&device_features)
            .push_next(&mut vk11_device_features);

        let device = unsafe {
            instance
                .create_device(device, &device_create_info, None)
                .expect("Failed to create logical device.")
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };

        (device, graphics_queue, present_queue)
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.loaders.surface_instance.destroy_surface(self.surface_khr, None);
            if let Some((utils, messenger)) = self.debug_report_callback.take() {
                utils.destroy_debug_utils_messenger(messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone, Copy)]
pub struct QueueFamilyIndices {
    pub graphics_index: u32,
    pub present_index: u32,
}

pub struct ExtensionLoaders {
    pub surface_instance: surface::Instance,
    pub swapchain_instance: swapchain::Instance,
    pub swapchain_device: swapchain::Device,
}
