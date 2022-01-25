use crate::GfxDevice;
use std::borrow::Cow;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, FragmentState,
    PipelineLayout, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureFormat, TextureSampleType, TextureViewDimension, VertexState,
};

pub struct GBufferPipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: RenderPipeline,
}

impl GBufferPipeline {
    pub fn new(device: &GfxDevice, output_format: TextureFormat) -> Self {
        let bind_group_layout = Self::create_bind_group_layout(device);
        let pipeline_layout = Self::create_pipeline_layout(device, &bind_group_layout);
        let pipeline = Self::create_pipeline(device, output_format, &pipeline_layout);
        Self {
            bind_group_layout,
            pipeline,
        }
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
        layout: &PipelineLayout,
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

    fn create_pipeline_layout(
        device: &GfxDevice,
        bind_group_layout: &BindGroupLayout,
    ) -> PipelineLayout {
        let layouts = [bind_group_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.device.create_pipeline_layout(&descriptor)
    }

    fn create_bind_group_layout(device: &GfxDevice) -> BindGroupLayout {
        let position_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: false },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
            visibility: ShaderStages::FRAGMENT,
        };

        let albedo_entry = BindGroupLayoutEntry {
            binding: 1,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
            visibility: ShaderStages::FRAGMENT,
        };

        let depth_entry = BindGroupLayoutEntry {
            binding: 2,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Depth,
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
            visibility: ShaderStages::FRAGMENT,
        };

        let sampler_entry = BindGroupLayoutEntry {
            binding: 3,
            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
            count: None,
            visibility: ShaderStages::FRAGMENT,
        };

        let depth_sampler_entry = BindGroupLayoutEntry {
            binding: 4,
            ty: BindingType::Sampler(SamplerBindingType::Comparison),
            count: None,
            visibility: ShaderStages::FRAGMENT,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                position_entry,
                albedo_entry,
                depth_entry,
                sampler_entry,
                depth_sampler_entry,
            ],
        };
        device.device.create_bind_group_layout(&descriptor)
    }
}
