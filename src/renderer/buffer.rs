use core::slice;
use std::{ffi::c_void, mem::size_of, ptr};

use ash::vk;

use super::{command_buffer::CommandBuffer, utility, vkcontext::VkContext};

pub struct Buffer<'ctx> {
    pub handle: vk::Buffer,
    pub device_memory: vk::DeviceMemory,
    pub size: u64,
    pub is_locked: bool,
    vkcontext: &'ctx VkContext,
}

impl<'ctx> Buffer<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        size: u64,
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        bind_on_create: bool,
    ) -> Self {
        let handle = {
            let create_info = vk::BufferCreateInfo::default()
                .size(size)
                .usage(buffer_usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            unsafe { vkcontext.device.create_buffer(&create_info, None).unwrap() }
        };

        let memory_properties = &vkcontext.physical_device_memory_properties;
        let memory_requirements = unsafe { vkcontext.device.get_buffer_memory_requirements(handle) };

        let memory_type = utility::query_memory_type(*memory_properties, memory_requirements, memory_property_flags);

        let device_memory = {
            let allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type.unwrap());

            unsafe { vkcontext.device.allocate_memory(&allocate_info, None).unwrap() }
        };

        if bind_on_create {
            unsafe {
                vkcontext.device.bind_buffer_memory(handle, device_memory, 0).unwrap()
            }
        }

        Self {
            handle,
            device_memory,
            size,
            is_locked: false,
            vkcontext,
        }
    }

    pub fn from_slice<T: Copy>(
        vkcontext: &'ctx VkContext,
        s: &[T],
        buffer_usage_flags: vk::BufferUsageFlags,
        memory_property_flags: vk::MemoryPropertyFlags,
        bind_on_create: bool,
    ) -> Self {
        let mut buffer = Self::new(
            vkcontext,
            (s.len() * std::mem::size_of::<T>()) as u64,
            buffer_usage_flags,
            memory_property_flags,
            bind_on_create
        );

        buffer.load_slice(0, s, vk::MemoryMapFlags::default());

        buffer
    }
}

impl<'ctx> Buffer<'ctx> {
    pub fn bind(&self, offset: vk::DeviceSize) {
        unsafe {
            self.vkcontext.device.bind_buffer_memory(self.handle, self.device_memory, offset).unwrap();
        }
    }

    pub fn lock_memory(
        &mut self,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        flags: vk::MemoryMapFlags
    ) -> *mut c_void {
        assert!(!self.is_locked);

        self.is_locked = true;

        unsafe {
            self.vkcontext.device.map_memory(self.device_memory, offset, size, flags).unwrap()
        }
    }

    pub fn unlock_memory(&mut self) {
        if !self.is_locked { return; }

        self.is_locked = false;

        unsafe {
            self.vkcontext.device.unmap_memory(self.device_memory);
        }
    }

    pub fn copy_to(
        &self,
        pool: vk::CommandPool,
        queue: vk::Queue,
        source_offset: vk::DeviceSize,
        dest: &mut Buffer,
        dest_offset: vk::DeviceSize,
        size: vk::DeviceSize,
    ) {
        unsafe { self.vkcontext.device.queue_wait_idle(queue).unwrap(); }

        let cb = CommandBuffer::new(self.vkcontext, pool, true);

        cb.begin(true, false, false);

        let buffer_copy = vk::BufferCopy::default()
            .src_offset(source_offset)
            .dst_offset(dest_offset)
            .size(size);

        unsafe {
            self.vkcontext.device.cmd_copy_buffer(cb.handle, self.handle, dest.handle, slice::from_ref(&buffer_copy));
        }

        cb.end_and_submit_single_use(queue);
    }

    pub fn upload_slice_staged<T: Copy>(
        &mut self,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        offset: vk::DeviceSize,
        s: &[T],
    ) {
        let flags = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let staging = Buffer::from_slice(
            self.vkcontext,
            s,
            vk::BufferUsageFlags::TRANSFER_SRC,
            flags,
            true,
        );

        staging.copy_to(
            command_pool, 
            queue, 
            0,
            self,
            offset,
            staging.size,
        );
    }
}

impl<'ctx> Buffer<'ctx> {
    pub fn load_value<T: Copy>(&mut self, offset: vk::DeviceSize, value: &T, flags: vk::MemoryMapFlags) {
        let buffer_adr = self.lock_memory(offset, size_of::<T>() as vk::DeviceSize, flags);

        unsafe { (buffer_adr as *mut T).copy_from_nonoverlapping(ptr::from_ref(value), 1); }

        self.unlock_memory();
    }

    pub fn load_slice<T: Copy>(&mut self, offset: vk::DeviceSize, s: &[T], flags: vk::MemoryMapFlags) {
        let buffer_adr = self.lock_memory(offset, (s.len() * size_of::<T>()) as vk::DeviceSize, flags);
        
        unsafe { (buffer_adr as *mut T).copy_from_nonoverlapping(s.as_ptr(), s.len()); }

        self.unlock_memory();
    }
}

impl<'ctx> Drop for Buffer<'ctx> {
    fn drop(&mut self) {
        unsafe {
            self.vkcontext.device.free_memory(self.device_memory, None);
            self.vkcontext.device.destroy_buffer(self.handle, None);
        }
    }
}
