use ash::vk;

use crate::math::vec3::Vec3F;

use super::{buffer::Buffer, vkcontext::VkContext};

struct Mesh<'ctx> {
    name: String,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    vertex_buffer: Buffer<'ctx>,
    index_buffer: Buffer<'ctx>,
}

impl<'ctx> Mesh<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        name: String,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let mut vertex_buffer = Buffer::new(
            vkcontext,
            (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            true,
        );

        let mut index_buffer = Buffer::new(
            vkcontext,
            (indices.len() * std::mem::size_of::<u32>()) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            true,
        );

        vertex_buffer.upload_slice_staged(command_pool, queue, 0, vertices);
        index_buffer.upload_slice_staged(command_pool, queue, 0, indices);

        Self {
            name,
            vertices: vertices.to_owned(),
            indices: indices.to_owned(),
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Clone, Copy)]
struct Vertex {
    position: Vec3F,
    texture_coordinate: Vec3F,
    normal: Vec3F,
}
