use crate::GfxDevice;
use std::borrow::Cow;
use wgpu::{
    FragmentState, RenderPipeline, TextureFormat, ShaderSource,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, VertexState
};

pub struct GBufferPipeline {
    pub pipeline: RenderPipeline,
}

impl GBufferPipeline {
    pub fn new(
        device: &GfxDevice,
        output_format: TextureFormat,
    ) -> Self {
        let pipeline = Self::create_pipeline(device, output_format);
        Self { pipeline }
    }

    fn create_shader(device: &GfxDevice) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("gbuf.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        device.device.create_shader_module(&descriptor)
    }

    fn create_pipeline(
        device: &GfxDevice,
        output_format: TextureFormat,
    ) -> RenderPipeline {
        let shader = Self::create_shader(device);

        let vertex = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        };

        let formats = [output_format.into()];
        let fragment = Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &formats,
        });


        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: None,
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
