use crate::BindGroups;

pub struct Pipelines {
    pub render: Vec<RenderPipeline>,
}

pub struct RenderPipeline {
    pub handle: wgpu::RenderPipeline,
    pub info: RenderPipelineInfo,
}

pub struct RenderPipelineInfo {
    pub shader: RenderShaderInfo,
    pub input: Option<RenderPipelineInput>,
    pub bind_groups: Vec<usize>,
    pub depth_test: bool,
    pub color_targets: Vec<wgpu::ColorTargetState>,
}

impl RenderPipelineInfo {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn depth_test(&self) -> Option<wgpu::DepthStencilState> {
        if !self.depth_test {
            return None;
        }

        Some(wgpu::DepthStencilState {
            format: Self::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    }

    pub fn primitive_state() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        }
    }
}

pub struct RenderShaderInfo {
    pub source: String,
    pub vs_main: String,
    pub fs_main: String,
}

pub struct RenderPipelineInput {
    pub vertex: InputBufferLayout,
    pub instance: InputBufferLayout,
}

pub struct InputBufferLayout {
    pub stride: u64,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl InputBufferLayout {
    pub fn raw_input_layout(&self) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: self.stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }
}

pub struct PipelinesFactory<'a> {
    device: &'a wgpu::Device,
    bind_groups: &'a BindGroups,
}

impl<'a> PipelinesFactory<'a> {
    pub fn new(device: &'a wgpu::Device, bind_groups: &'a BindGroups) -> Self {
        Self {
            device,
            bind_groups,
        }
    }

    pub fn create_render_pipeline(&self, info: RenderPipelineInfo) -> RenderPipeline {
        let shader_module = self.create_shader_module(&info.shader);

        let input_layouts = info.input.as_ref().map(|input| {
            [
                input.vertex.raw_input_layout(),
                input.instance.raw_input_layout(),
            ]
        });
        let input_layouts = match input_layouts.as_ref() {
            Some(arr) => arr.as_slice(),
            None => [].as_slice(),
        };

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: &info.shader.vs_main,
            buffers: &input_layouts,
        };

        let layout = self.create_pipeline_layout(&info.bind_groups);
        let primitive = RenderPipelineInfo::primitive_state();
        let depth_stencil = info.depth_test();

        let fragment = Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: &info.shader.fs_main,
            targets: &info.color_targets,
        });

        let descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex,
            primitive,
            depth_stencil,
            multisample: Default::default(),
            fragment,
            multiview: None,
        };

        let handle = self.device.create_render_pipeline(&descriptor);
        RenderPipeline { handle, info }
    }

    fn create_shader_module(&self, shader: &RenderShaderInfo) -> wgpu::ShaderModule {
        use std::borrow::Cow;

        let shader_src = Cow::Borrowed(shader.source.as_str());
        let source = wgpu::ShaderSource::Wgsl(shader_src);
        let descriptor = wgpu::ShaderModuleDescriptor {
            label: None,
            source,
        };
        self.device.create_shader_module(&descriptor)
    }

    fn create_pipeline_layout(&self, bind_group_indices: &[usize]) -> wgpu::PipelineLayout {
        let layouts: Vec<_> = bind_group_indices
            .iter()
            .map(|index| &self.bind_groups.0[*index].layout)
            .collect();

        let descriptor = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };

        self.device.create_pipeline_layout(&descriptor)
    }
}
