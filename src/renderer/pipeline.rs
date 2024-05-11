use ash::vk;
use super::vkcontext::VkContext;

pub struct Pipeline<'c> {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    vkcontext: &'c VkContext,
}

impl<'c> Pipeline<'c> {
    pub fn new_graphics(
        vkcontext: &'c VkContext,
        render_pass: vk::RenderPass,
        subpass_index: u32,
        pipeline_state_info: &PipelineStateInfo,
        vertex_bindings: &[vk::VertexInputBindingDescription],
        vertex_attributes: &[vk::VertexInputAttributeDescription],
        push_constant_ranges: &[vk::PushConstantRange],
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
        color_blend_attachment_states: &[vk::PipelineColorBlendAttachmentState],
        depth_test_enabled: bool,
    ) -> Self {
        for state in Self::REQUIRED_DYNAMIC_STATE {
            if !pipeline_state_info.dynamic_state.contains(&state) {
                panic!("Dynamic state supplied to Pipeline is missing one or more required dynamic states.");
            }
        }

        let layout = {
            let create_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(descriptor_set_layouts)
                .push_constant_ranges(push_constant_ranges);

            unsafe { vkcontext.device.create_pipeline_layout(&create_info, None).unwrap() }
        };
        
        let handle = {
            let input_state = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(vertex_bindings)
                .vertex_attribute_descriptions(vertex_attributes);

            let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(color_blend_attachment_states);

            let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(pipeline_state_info.dynamic_state);

            // TODO: Allow tesselation state.
            let mut create_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(shader_stages)
                .vertex_input_state(&input_state)
                .input_assembly_state(&pipeline_state_info.input_assembly_state)
                .viewport_state(&pipeline_state_info.viewport_state)
                .rasterization_state(&pipeline_state_info.rasterizer_state)
                .multisample_state(&pipeline_state_info.multisampler_state)
                .color_blend_state(&color_blend)
                .dynamic_state(&dynamic_state)
                
                .layout(layout)

                .render_pass(render_pass)
                .subpass(subpass_index);
                
            if depth_test_enabled {
                create_info = create_info.depth_stencil_state(&pipeline_state_info.depth_stencil_state);
            }

            unsafe {
                vkcontext.device.create_graphics_pipelines(
                    vk::PipelineCache::default(),
                    std::slice::from_ref(&create_info),
                    None
                )
                .unwrap()[0]
            }
        };

        Self {
            handle,
            layout,
            vkcontext,
        }
    }

    pub fn new_compute(
        vkcontext: &'c VkContext,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        compute_stage_create_info: vk::PipelineShaderStageCreateInfo,
    ) -> Self {


        let layout = { 
            let create_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(descriptor_set_layouts);

            unsafe { vkcontext.device.create_pipeline_layout(&create_info, None).unwrap() }
        };


        let handle = {
            let create_info = vk::ComputePipelineCreateInfo::default()
                .stage(compute_stage_create_info)
                .layout(layout);

            let create_infos = [create_info];
            
            unsafe {
                vkcontext.device
                .create_compute_pipelines(vk::PipelineCache::null(), &create_infos, None)
                .unwrap()[0]
            }
        };

        Self {
            handle,
            layout,
            vkcontext,
        }
    }
}

impl<'c> Pipeline<'c> {
    pub const REQUIRED_DYNAMIC_STATE: [vk::DynamicState; 3] = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
        vk::DynamicState::LINE_WIDTH,
    ];
}

impl<'c> Drop for Pipeline<'c> {
    fn drop(&mut self) {
        unsafe {
            self.vkcontext.device.destroy_pipeline_layout(self.layout, None);
            self.vkcontext.device.destroy_pipeline(self.handle, None);
        }
    }
}

pub struct PipelineStateInfo<'a> {
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    rasterizer_state: vk::PipelineRasterizationStateCreateInfo<'a>,
    multisampler_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    dynamic_state: &'a [vk::DynamicState],
}

impl<'a> PipelineStateInfo<'a> {
    pub const DEFAULT_DYNAMIC_STATE: [vk::DynamicState; 3] = [
        vk::DynamicState::VIEWPORT,
        vk::DynamicState::SCISSOR,
        vk::DynamicState::LINE_WIDTH,
    ];

    pub fn get_default_pipeline_state_info() -> Self {
        Self {
            viewport_state: vk::PipelineViewportStateCreateInfo::default(),
            input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false),
            rasterizer_state: vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0f32)
                .depth_bias_clamp(0f32)
                .depth_bias_slope_factor(0f32)
                .line_width(1f32),
            multisampler_state: vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .sample_shading_enable(false)
                .min_sample_shading(1f32)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false),
            depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false),
            dynamic_state: &Self::DEFAULT_DYNAMIC_STATE,
        }
    }
}
