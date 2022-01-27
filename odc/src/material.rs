use crate::{DrawData, GBuffer, GfxDevice, Uniform};
use core::ops::Range;
use wgpu::{
    BindGroup, CompareFunction, DepthBiasState, DepthStencilState, FragmentState, PipelineLayout,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, StencilState, VertexAttribute,
    VertexBufferLayout, VertexState, VertexStepMode,
};

pub struct Material {
    pipeline: RenderPipeline,
    uniform_bind_group: BindGroup,
}

impl Material {
    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>, to_draw: &[DrawData]) {
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_pipeline(&self.pipeline);
        for draw in to_draw.iter() {
            pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instances.clone(),
            )
        }
    }
}

pub struct MaterialInfo<'a> {
    pub vertex: InputInfo<'a>,
    pub instance: InputInfo<'a>,
    pub shader: ShaderInfo<'a>,
    pub uniform_location: Range<u64>,
}

pub struct ShaderInfo<'a> {
    pub source: &'a str,
    pub vs_entry: &'a str,
    pub fs_entry: &'a str,
}

pub struct InputInfo<'a> {
    pub attributes: &'a [InputAttribute],
    pub stride: u64,
}

pub type InputAttribute = VertexAttribute;

pub struct MaterialFactory<'a> {
    pub device: &'a GfxDevice,
    pub layout: &'a PipelineLayout,
    pub uniform: &'a Uniform,
}

impl<'a> MaterialFactory<'a> {
    pub fn create_material(&self, info: &MaterialInfo) -> Material {
        let pipeline = self.create_pipeline(info);
        let uniform_bind_group = self
            .uniform
            .create_bind_group(self.device, &info.uniform_location);
        Material {
            pipeline,
            uniform_bind_group,
        }
    }

    fn vertex_layout(info: &'a InputInfo) -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: info.stride,
            step_mode: VertexStepMode::Vertex,
            attributes: info.attributes,
        }
    }

    fn instance_layout(info: &'a InputInfo) -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: info.stride,
            step_mode: VertexStepMode::Instance,
            attributes: info.attributes,
        }
    }

    fn create_pipeline(&self, info: &'a MaterialInfo) -> RenderPipeline {
        let vertex_layout = Self::vertex_layout(&info.vertex);
        let instance_layout = Self::instance_layout(&info.instance);

        let shader = self.device.create_shader(info.shader.source);

        let vertex = VertexState {
            module: &shader,
            entry_point: info.shader.vs_entry,
            buffers: &[vertex_layout, instance_layout],
        };

        let formats = [
            GBuffer::POSITION_FORMAT.into(),
            GBuffer::NORMAL_FORMAT.into(),
            GBuffer::ALBEDO_FORMAT.into(),
        ];
        let fragment = Some(FragmentState {
            module: &shader,
            entry_point: info.shader.fs_entry,
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
            layout: Some(self.layout),
            vertex,
            fragment,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: Some(depth_stencil_state),
            multiview: None,
        };

        self.device.device.create_render_pipeline(&descriptor)
    }
}
