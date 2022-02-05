use crate::model as mdl;
use crate::{Resources, Swapchain};
use raw_window_handle::HasRawWindowHandle;

pub struct WindowInfo<'a, Handle: HasRawWindowHandle> {
    pub handle: &'a Handle,
    pub size: mdl::Size2d,
}

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
        let pipeline = Self::create_pipeline(device, swapchain.format, pipeline_layout, resources, source_id);

        let bind_group = Self::create_bind_group(device, &layout, resources, source_id);

        Self {
            swapchain,
            layout,
            pipeline,
            bind_group,
        }
    }

    pub fn refresh_bind_group(
        &mut self,
        device: &wgpu::Device,
        resources: &Resources,
        source_id: &str,
    ) {
        let bind_group = Self::create_bind_group(device, &self.layout, resources, source_id);
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
        name: &str,
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

        let sampler_binding_type = Resources::sampler_binding_type_from_format(texture_format);
        let sampler_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(sampler_binding_type),
            count: None,
        };

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some(name),
            entries: &[texture_entry, sampler_entry],
        };
        device.create_bind_group_layout(&descriptor)
    }

    fn create_pipeline(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        layout: wgpu::PipelineLayout,
        resources: &Resources,
        name: &str,
    ) -> wgpu::RenderPipeline {
        let attachment_format = resources.texture_format(name);
        let descriptor = match attachment_format.describe().sample_type {
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

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resources: &Resources,
        name: &str,
    ) -> wgpu::BindGroup {
        let view = resources.texture_view(name);
        let texture_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&view),
        };

        let sampler_type = resources.texture_sampler_type(name);
        println!("using sampler type in window: {:?}", sampler_type);
        let sampler = resources.raw_sampler(sampler_type);
        let sampler_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(sampler),
        };
        let descriptor = wgpu::BindGroupDescriptor {
            label: Some(name),
            layout,
            entries: &[texture_entry, sampler_entry],
        };

        device.create_bind_group(&descriptor)
    }
}
