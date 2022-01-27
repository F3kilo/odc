use crate::instances::Instances;
use crate::uniform::Uniform;
use crate::GBuffer;
use crate::GfxDevice;
use std::borrow::Cow;
use std::mem;
use wgpu::{
    BindGroupLayout, CompareFunction, DepthBiasState, DepthStencilState, FragmentState,
    PipelineLayout, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, StencilState, VertexBufferLayout,
    VertexState, VertexStepMode,
};

pub struct ColorMeshPipeline {
    pub pipeline: RenderPipeline,
}

impl ColorMeshPipeline {
    pub fn new(device: &GfxDevice, instances: &Instances, uniform: &Uniform) -> Self {
        let pipeline_layout = Self::create_layout(device, &instances.layout, &uniform.layout);
        let pipeline = Self::create_pipeline(device, &pipeline_layout);

        Self { pipeline }
    }

    fn create_shader(device: &GfxDevice) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("color_mesh.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        device.device.create_shader_module(&descriptor)
    }

    fn create_layout(
        device: &GfxDevice,
        instances_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> PipelineLayout {
        let layouts = [instances_layout, uniform_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.device.create_pipeline_layout(&descriptor)
    }

    fn create_pipeline(device: &GfxDevice, layout: &PipelineLayout) -> RenderPipeline {
        const FLOAT_SIZE: u64 = mem::size_of::<f32>() as _;
        let attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];

        let vertex_layout = VertexBufferLayout {
            array_stride: 12 * FLOAT_SIZE,
            attributes: &attributes,
            step_mode: VertexStepMode::Vertex,
        };

        let shader = Self::create_shader(device);

        let vertex = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout],
        };

        let formats = [
            GBuffer::POSITION_FORMAT.into(),
            GBuffer::NORMAL_FORMAT.into(),
            GBuffer::ALBEDO_FORMAT.into(),
        ];
        let fragment = Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &formats,
        });

        let depth_stencil_state = DepthStencilState {
            format: GBuffer::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        };

        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex,
            fragment,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: Some(depth_stencil_state),
            multiview: None,
        };

        device.device.create_render_pipeline(&descriptor)
    }
}
