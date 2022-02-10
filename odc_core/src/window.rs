use crate::{mdl, Swapchain};
use raw_window_handle::HasRawWindowHandle;
use wgpu::TextureSampleType;

pub struct WindowInfo<'a, Handle: HasRawWindowHandle> {
    pub name: &'a str,
    pub handle: &'a Handle,
    pub size: mdl::Size2d,
}

pub struct WindowSource {
    pub texture_view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

pub struct Window {
    sampler: Sampler,
    swapchain: Swapchain,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Window {
    pub fn new(device: &wgpu::Device, swapchain: Swapchain, source: WindowSource) -> Self {
        let sampler = Sampler::new(device, source.format);
        let layout = Self::create_bind_group_layout(device, source.format);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline =
            Self::create_pipeline(device, source.format, swapchain.format, pipeline_layout);

        let bind_group =
            Self::create_bind_group(device, &layout, &source.texture_view, &sampler.handle);

        Self {
            sampler,
            swapchain,
            layout,
            pipeline,
            bind_group,
        }
    }

    pub fn refresh_bind_group(&mut self, device: &wgpu::Device, source_view: &wgpu::TextureView) {
        let bind_group =
            Self::create_bind_group(device, &self.layout, source_view, &self.sampler.handle);
        self.bind_group = bind_group;
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) -> Option<wgpu::SurfaceTexture> {
        let frame = match self.swapchain.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Outdated) => return None,
            e => e.unwrap(),
        };

        let view = frame.texture.create_view(&Default::default());
        let attachment = wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: true,
            },
        };

        let descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[attachment],
            depth_stencil_attachment: None,
        };
        let mut pass = encoder.begin_render_pass(&descriptor);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
        Some(frame)
    }

    pub fn resize(&self, device: &wgpu::Device, size: mdl::Size2d) {
        self.swapchain.resize(device, size)
    }

    fn create_bind_group_layout(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> wgpu::BindGroupLayout {
        let sample_type = texture_format.describe().sample_type;
        println!("PipelineLayout sample type: {:?}", sample_type);
        let ty = wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };

        let texture_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };

        let sampler_binding_type = match texture_format.describe().sample_type {
            wgpu::TextureSampleType::Float { filterable: false } => {
                wgpu::SamplerBindingType::NonFiltering
            }
            _ => wgpu::SamplerBindingType::Filtering,
        };

        let sampler_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(sampler_binding_type),
            count: None,
        };

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[texture_entry, sampler_entry],
        };
        device.create_bind_group_layout(&descriptor)
    }

    fn create_pipeline(
        device: &wgpu::Device,
        source_format: wgpu::TextureFormat,
        target_format: wgpu::TextureFormat,
        layout: wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let descriptor = match source_format.describe().sample_type {
            wgpu::TextureSampleType::Depth => {
                println!("using depth window shader");
                wgpu::include_wgsl!("../data/shaders/window_depth.wgsl")
            }
            _ => wgpu::include_wgsl!("../data/shaders/window.wgsl"),
        };

        let shader_module = device.create_shader_module(&descriptor);

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[],
        };

        let color_targets = [wgpu::ColorTargetState {
            format: target_format,
            blend: None,
            write_mask: Default::default(),
        }];

        let fragment = Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &color_targets,
        });

        let descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex,
            fragment,
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        };

        device.create_render_pipeline(&descriptor)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        let texture_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(view),
        };

        let sampler_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(sampler),
        };
        let descriptor = wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[texture_entry, sampler_entry],
        };

        device.create_bind_group(&descriptor)
    }
}

pub struct Sampler {
    handle: wgpu::Sampler,
}

impl Sampler {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let sample_type = texture_format.describe().sample_type;

        let filter_mode = match sample_type {
            TextureSampleType::Float { filterable: false } => wgpu::FilterMode::Nearest,
            _ => wgpu::FilterMode::Linear,
        };

        let descriptor = wgpu::SamplerDescriptor {
            min_filter: filter_mode,
            mag_filter: filter_mode,
            mipmap_filter: filter_mode,
            ..Default::default()
        };

        let handle = device.create_sampler(&descriptor);
        Self { handle }
    }
}
