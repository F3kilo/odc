use crate::{Resources, Swapchain};

pub struct Window {
    swapchain: Swapchain,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Window {
    pub fn new(
        device: &wgpu::Device,
        swapchain: Swapchain,
        resources: &Resources,
        source_id: &str,
    ) -> Self {
        let texture_format = resources.texture_format(source_id);
        let layout = Self::create_bind_group_layout(device, texture_format, source_id);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(source_id),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(device, swapchain.format, pipeline_layout, source_id);

        let view = resources.texture_view(source_id);
        let bind_group = Self::create_bind_group(device, &layout, &view, source_id);

        Self {
            swapchain,
            layout,
            pipeline,
            bind_group,
        }
    }

    pub fn create_bind_group_layout(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        name: &str,
    ) -> wgpu::BindGroupLayout {
        let sample_type = texture_format.describe().sample_type;

        let ty = wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        };

        let entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some(name),
            entries: &[entry],
        };
        device.create_bind_group_layout(&descriptor)
    }

    pub fn create_pipeline(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        layout: wgpu::PipelineLayout,
        name: &str,
    ) -> wgpu::RenderPipeline {
        let descriptor = wgpu::include_wgsl!("../data/shaders/window.wgsl");
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
            label: Some(name),
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

    pub fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        name: &str,
    ) -> wgpu::BindGroup {
        let entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(view),
        };

        let descriptor = wgpu::BindGroupDescriptor {
            label: Some(name),
            layout,
            entries: &[entry],
        };

        device.create_bind_group(&descriptor)
    }
}
