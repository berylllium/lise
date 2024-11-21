use ash::vk;

use crate::math::{vec2::Vec2F, vec3::Vec3F};

use super::{buffer::Buffer, vkcontext::VkContext};

pub struct Mesh<'ctx> {
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
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        let mut vertex_buffer = Buffer::new(
            vkcontext,
            std::mem::size_of_val(vertices) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            true,
        );

        let mut index_buffer = Buffer::new(
            vkcontext,
            std::mem::size_of_val(indices) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            true,
        );

        vertex_buffer.upload_slice_staged(command_pool, queue, 0, vertices);
        index_buffer.upload_slice_staged(command_pool, queue, 0, indices);

        Self {
            vertices: vertices.to_owned(),
            indices: indices.to_owned(),
            vertex_buffer,
            index_buffer,
        }
    }
}

impl<'ctx> Mesh<'ctx> {
    pub fn draw(&self) {
        
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3F,
    pub texture_coordinate: Vec2F,
    pub normal: Vec3F,
}

impl Vertex {
    pub fn get_binding_description(binding: u32) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(binding)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
}
