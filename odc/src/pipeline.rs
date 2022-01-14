use crate::instances::Instances;
use crate::uniform::Uniform;
use crate::{GfxDevice, Vertex};
use std::borrow::Cow;
use wgpu::{
    BindGroupLayout, FragmentState, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

pub struct ColorMeshPipeline {
    pub pipeline: RenderPipeline,
}

impl ColorMeshPipeline {
    pub fn new(
        device: &GfxDevice,
        instances: &Instances,
        uniform: &Uniform,
        format: TextureFormat,
    ) -> Self {
        let pipeline_layout = Self::create_layout(device, &instances.layout, &uniform.layout);
        let pipeline = Self::create_pipeline(device, &pipeline_layout, format);

        Self { pipeline }
    }
    fn create_shader(device: &GfxDevice) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("shader.wgsl"));
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

    fn create_pipeline(
        device: &GfxDevice,
        layout: &PipelineLayout,
        output_format: TextureFormat,
    ) -> RenderPipeline {
        let attributes = [
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::position_offset() as _,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::color_offset() as _,
                shader_location: 1,
            },
        ];

        let vertex_layout = VertexBufferLayout {
            array_stride: Vertex::size() as _,
            attributes: &attributes,
            step_mode: VertexStepMode::Vertex,
        };

        let shader = Self::create_shader(device);

        let vertex = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout],
        };

        let formats = [output_format.into()];
        let fragment = Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &formats,
        });

        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex,
            fragment,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: None,
            multiview: None,
        };

        device.device.create_render_pipeline(&descriptor)
    }
}
