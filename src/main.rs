use std::mem::size_of;

use ash::vk::{self, AttachmentDescription, SubpassDependency};
use lise::{math::vec2::Vec2UI, node::Node, renderer::{self, frame_buffer::Framebuffer, render_pass::{RenderPass, RenderPassSubPassInfo}, shader::{Shader, ShaderDescriptorInfo, ShaderDescriptorSetInfo, ShaderDescriptorTypeInfo, ShaderPushConstantInfo, ShaderStageInfo, ShaderType, ShaderVertexAttributeInfo}, vkcontext::VkContext, Renderer}, utility::Clock};
use simple_logger::SimpleLogger;
use simple_window::{Window, WindowEvent};

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut window = Window::new("LiSE Test", 200, 200, 400, 500);

    let vkcontext = VkContext::new(&window);

    let mut renderer = Renderer::new(&vkcontext);

    let world_render_pass = RenderPass::new(
        &vkcontext,
        Vec2UI::default(),
        renderer.get_render_area_size(),
        &[
            AttachmentDescription::default()
                .format(renderer.swapchain.swapchain_properties.format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
        ],
        &[
            Some(vk::ClearValue { color: vk::ClearColorValue { float32: [0.4f32, 0.5f32, 0.6f32, 0f32] } }),
        ],
        &[
            RenderPassSubPassInfo {
                bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachments: &[],
                color_attachments: Some(&[
                    vk::AttachmentReference {
                        attachment: 0,
                        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    }
                ]),
                resolve_attachments: None,
                depth_stencil_attachments: None,
                preserve_attachments: None,
            },
        ],
        &[
            SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                dst_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::default(),
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dependency_flags: vk::DependencyFlags::default(),
            }
        ]
    );

    let framebuffers = (0..renderer.swapchain.image_views.len()).map(|i| {
        let attachments = [renderer.swapchain.image_views[i]];

        Framebuffer::new(&vkcontext, world_render_pass.handle, &attachments, renderer.get_render_area_size())
    })
    .collect::<Vec<_>>();

    let mesh_shader = Shader::new(
        &vkcontext,
        "LiSE Test",
        world_render_pass.handle,
        0,
        &[
            vk::PipelineColorBlendAttachmentState {
                blend_enable: vk::TRUE,
                src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                color_blend_op: vk::BlendOp::ADD,
                src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
                dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
                alpha_blend_op: vk::BlendOp::ADD,
                color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            },
        ],
        &[ Vertex::get_binding_description(0) ],
        &[
            ShaderVertexAttributeInfo { attribute_type: ShaderType::Float32_3, binding: 0 },
            ShaderVertexAttributeInfo { attribute_type: ShaderType::Float32_3, binding: 0 },
            ShaderVertexAttributeInfo { attribute_type: ShaderType::Float32_2, binding: 0 },
        ],
        &[
            ShaderPushConstantInfo { push_constant_type: ShaderType::Matrix4, stage_flags: vk::ShaderStageFlags::VERTEX },
        ],
        &[
            ShaderDescriptorSetInfo {
                max_set_allocations: 1 * renderer::MAX_FRAMES_IN_FLIGHT,
                descriptors: &[
                    ShaderDescriptorInfo {
                        descriptor_type: ShaderDescriptorTypeInfo::UniformBuffer { 
                            fields: &[ ShaderType::Matrix4, ShaderType::Matrix4 ],
                        },
                        stage_flags: vk::ShaderStageFlags::VERTEX,
                    },
                ]
            },
            ShaderDescriptorSetInfo {
                max_set_allocations: 1000 * renderer::MAX_FRAMES_IN_FLIGHT,
                descriptors: &[
                    ShaderDescriptorInfo {
                        descriptor_type: ShaderDescriptorTypeInfo::UniformBuffer { 
                            fields: &[ ShaderType::Float32_4 ],
                        },
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    },
                    ShaderDescriptorInfo {
                        descriptor_type: ShaderDescriptorTypeInfo::Sampler,
                        stage_flags: vk::ShaderStageFlags::FRAGMENT,
                    },
                ],
            },
        ],
        &[
            ShaderStageInfo {
                stage_type: vk::ShaderStageFlags::VERTEX,
                stage_file: "shaders/builtin.meshshader.vert.spv",
            },
            ShaderStageInfo {
                stage_type: vk::ShaderStageFlags::FRAGMENT,
                stage_file: "shaders/builtin.meshshader.frag.spv",
            },
        ],
        false,
    );

    // Node testing.
    let mut root = Node::new("Root", None);
    root.add_child(Node::new("C1", None));
    root.add_child(Node::new("C2", None));
    root.add_child(Node::new("C3", None));

    for node in root.iter() {
        log::debug!("Node: {}", node.name);
    }

    // Loop.
    
    let mut clock = Clock::new();
    let mut sum_time = 0u32;
    let mut frame_sum = 0u32;

    log::debug!("Entering game loop.");
    let mut is_running = true;

    while is_running {
        clock.reset();

        window.poll_messages(|event| {
            if let WindowEvent::Close = event {
                is_running = false;
            }
        });

        if sum_time >= 1000000 {
            log::debug!("It's been {} microseconds. {} frames have elapsed. FPS: {}", sum_time, frame_sum, frame_sum as f32 / (sum_time as f32 / 1000000f32));
            sum_time = 0;
            frame_sum = 0;
        }
        
        renderer.prepare_frame();

        world_render_pass.begin(renderer.get_current_command_buffer_handle(), framebuffers[renderer.current_image_index as usize].handle);

        mesh_shader.bind(renderer.get_current_command_buffer_handle());

        world_render_pass.end(renderer.get_current_command_buffer_handle());

        renderer.submit_frame();
        sum_time += clock.elapsed() as u32;
        frame_sum += 1;
    }

    unsafe {
        vkcontext.device.device_wait_idle().unwrap();
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    fn get_binding_description(binding: u32) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(binding)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
}
