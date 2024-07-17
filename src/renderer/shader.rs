use std::{ffi::CString, marker::PhantomData, ptr};

use ash::vk;

use super::{pipeline::{Pipeline, PipelineStateInfo}, vkcontext::VkContext};

pub struct Shader<'ctx> {
    pub name: String,
    pub minimum_uniform_alignment: u64,
    
    pub descriptor_pool: vk::DescriptorPool,

    pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,

    pub pipeline: Pipeline<'ctx>,

    vkcontext: &'ctx VkContext,
}

impl<'ctx> Shader<'ctx> {
    pub fn new(
        vkcontext: &'ctx VkContext,
        name: &str,
        render_pass: vk::RenderPass,
        subpass_index: u32,
        color_blend_attachment_states: &[vk::PipelineColorBlendAttachmentState],
        vertex_bindings: &[vk::VertexInputBindingDescription],
        vertex_attributes: &[ShaderVertexAttributeInfo],
        push_constants: &[ShaderPushConstantInfo],
        descriptor_sets: &[ShaderDescriptorSetInfo],
        shader_stages: &[ShaderStageInfo],
        depth_test_enabled: bool,
    ) -> Self {
        // Create Shader Stages.
        let shader_stages = shader_stages.iter().map(|stage| {
            ShaderStage::new(vkcontext, stage.stage_file, stage.stage_type)
        })
        .collect::<Vec<_>>();

        // Vertex attributes.
        let mut vertex_attribute_offset = 0u32;

        let vertex_attributes = vertex_attributes.iter().enumerate().map(|(i, attrib)| {
            let vertex_attribute = vk::VertexInputAttributeDescription::default()
                .binding(attrib.binding)
                .location(i as u32)
                .format(attrib.attribute_type.as_vk_format())
                .offset(vertex_attribute_offset);

            vertex_attribute_offset += attrib.attribute_type.size();

            vertex_attribute
        })
        .collect::<Vec<_>>();

        // Descriptors.
        let descriptor_set_layouts = descriptor_sets.iter().map(|set_info| {
            let layout_bindings = set_info.descriptors.iter().enumerate().map(|(i, descriptor)| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(i as u32)
                    .descriptor_type(descriptor.descriptor_type.as_vk_descriptor_type())
                    .descriptor_count(1)
                    .stage_flags(descriptor.stage_flags)
            })
            .collect::<Vec<_>>();

            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&layout_bindings);

            unsafe { vkcontext.device.create_descriptor_set_layout(&create_info, None).unwrap() }
        })
        .collect::<Vec<_>>();

        let mut pool_sizes = Vec::new();
        let mut max_pool_set_count = 0u32;

        for set_info in descriptor_sets {
            pool_sizes.extend(set_info.descriptors.iter().map(|descriptor| {
                vk::DescriptorPoolSize::default()
                    .ty(descriptor.descriptor_type.as_vk_descriptor_type())
                    .descriptor_count(set_info.max_set_allocations)
            }));

            max_pool_set_count += set_info.max_set_allocations;
        }

        let descriptor_pool = {
            let ci = vk::DescriptorPoolCreateInfo::default()
                .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
                .max_sets(max_pool_set_count)
                .pool_sizes(&pool_sizes);

            unsafe { vkcontext.device.create_descriptor_pool(&ci, None).unwrap() }
        };

        let mut push_constant_offset = 0u32;
        let push_constant_ranges = push_constants.iter().map(|push_constant| {
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(push_constant.stage_flags)
                .offset(push_constant_offset)
                .size(push_constant.push_constant_type.size());

            push_constant_offset += push_constant.push_constant_type.size();

            push_constant_range
        })
        .collect::<Vec<_>>();

        // Pipeline.
        let pipeline = Pipeline::new_graphics(
            vkcontext,
            render_pass,
            subpass_index,
            &PipelineStateInfo::get_default_pipeline_state_info(),
            vertex_bindings,
            &vertex_attributes,
            &push_constant_ranges,
            &descriptor_set_layouts,
            &shader_stages.iter().map(|stage| stage.shader_stage_create_info).collect::<Vec<_>>(),
            color_blend_attachment_states,
            depth_test_enabled,
        );

        Self {
            name: name.to_string(),
            minimum_uniform_alignment: vkcontext.physical_device_properties.limits.min_uniform_buffer_offset_alignment,
            descriptor_pool,
            descriptor_set_layouts,
            pipeline,
            vkcontext,
        }
    }
}

impl<'ctx> Shader<'ctx> {
    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        self.pipeline.bind(command_buffer, vk::PipelineBindPoint::GRAPHICS);
    }
}

impl<'ctx> Drop for Shader<'ctx> {
    fn drop(&mut self) {
        unsafe {
            self.vkcontext.device.destroy_descriptor_pool(self.descriptor_pool, None);
            
            for descriptor_set_layout in self.descriptor_set_layouts.iter() {
                self.vkcontext.device.destroy_descriptor_set_layout(*descriptor_set_layout, None);
            }
        }
    }
}

struct ShaderAttribute {
    name: String,
    format: vk::Format,
    size: u32,
}

struct ShaderStage<'ctx, 'a> {
    module: vk::ShaderModule,
    shader_stage_create_info: vk::PipelineShaderStageCreateInfo<'a>,
    stage_entry_point_name: CString,
    vkcontext: &'ctx VkContext,
}

impl<'ctx, 'a> ShaderStage<'ctx, 'a> {
    fn new<P: AsRef<std::path::Path>>(vkcontext: &'ctx VkContext, path: P, shader_stage: vk::ShaderStageFlags) -> Self {
        let compute_code = read_shader_from_file(path);

        let module = {
            let create_info = vk::ShaderModuleCreateInfo::default()
                .code(&compute_code);

            unsafe { vkcontext.device.create_shader_module(&create_info, None).unwrap() }
        };

        let entry_point_name = CString::new("main").unwrap();

        let shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::default(),
            stage: shader_stage,
            module,
            p_name: entry_point_name.as_ptr(),
            p_specialization_info: ptr::null(),
            _marker: PhantomData,
        };

        Self {
            module,
            shader_stage_create_info,
            stage_entry_point_name: entry_point_name,
            vkcontext,
        }
    }
}

impl<'ctx, 'a> Drop for ShaderStage<'ctx, 'a> {
    fn drop(&mut self) {
        unsafe {
            self.vkcontext.device.destroy_shader_module(self.module, None);
        }
    }
}

pub struct ShaderStageInfo<'a> {
    pub stage_type: vk::ShaderStageFlags,
    pub stage_file: &'a str,
}

pub struct ShaderPushConstantInfo {
    pub push_constant_type: ShaderType,
    pub stage_flags: vk::ShaderStageFlags,
}

pub struct ShaderVertexAttributeInfo {
    pub attribute_type: ShaderType,
    pub binding: u32,
}

pub struct ShaderDescriptorSetInfo<'a> {
    pub max_set_allocations: u32,
    pub descriptors: &'a [ShaderDescriptorInfo<'a>],
}

pub struct ShaderDescriptorInfo<'a> {
    pub descriptor_type: ShaderDescriptorTypeInfo<'a>,
    pub stage_flags: vk::ShaderStageFlags,
}

pub enum ShaderDescriptorTypeInfo<'a> {
    UniformBuffer { fields: &'a [ShaderType] },
    Sampler
}

impl<'a> ShaderDescriptorTypeInfo<'a> {
    pub fn as_vk_descriptor_type(&self) -> vk::DescriptorType {
        match self {
            Self::UniformBuffer { .. } => vk::DescriptorType::UNIFORM_BUFFER,
            Self::Sampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        }
    }
}

pub enum ShaderType {
    Float32,
    Float32_2,
    Float32_3,
    Float32_4,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Matrix4,
    Sampler,
}

impl ShaderType {
    pub fn size(&self) -> u32 {
        match self {
            Self::Int8 | Self::UInt8 => 1,
            Self::Int16 | Self::UInt16 => 2,
            Self::Float32 | Self::Int32 | Self::UInt32 => 4,
            Self::Float32_2 => 8,
            Self::Float32_3 => 12,
            Self::Float32_4 => 16,
            Self::Matrix4 => 64,
            Self::Sampler => 0,
        }
    }

    pub fn as_vk_format(&self) -> vk::Format {
        match self {
            Self::Float32 => vk::Format::R32_SFLOAT,
            Self::Float32_2 => vk::Format::R32G32_SFLOAT,
            Self::Float32_3 => vk::Format::R32G32B32_SFLOAT,
            Self::Float32_4 => vk::Format::R32G32B32A32_SFLOAT,
            Self::Int8 => vk::Format::R8_SINT,
            Self::UInt8 => vk::Format::R8_UINT,
            Self::Int16 => vk::Format::R16_SINT,
            Self::UInt16 => vk::Format::R16_UINT,
            Self::Int32 => vk::Format::R32_SINT,
            Self::UInt32 => vk::Format::R32_UINT,
            Self::Matrix4 | Self::Sampler => panic!("Provided ShaderType has no valid vk::Format."),
        }
    }
}

fn read_shader_from_file<P: AsRef<std::path::Path>>(path: P) -> Vec<u32> {
    use crate::utility::fs;

    log::debug!("Reading shader file: {}", path.as_ref().to_str().unwrap());

    let mut cursor = fs::load(path);

    ash::util::read_spv(&mut cursor).unwrap()
}
