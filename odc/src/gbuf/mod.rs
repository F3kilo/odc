pub mod pipeline;

use crate::gdevice::GfxDevice;
use crate::WindowSize;
use pipeline::GBufferPipeline;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Color,
    CommandEncoder, Extent3d, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    Sampler, SamplerDescriptor, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
};

pub struct GBuffer {
    textures: Textures,
    sampler: Sampler,
    depth_sampler: Sampler,
    gbuf_pipeline: GBufferPipeline,
    bind_group: BindGroup,
}

impl GBuffer {
    pub const POSITION_FORMAT: TextureFormat = TextureFormat::Rg32Float;
    pub const ALBEDO_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    pub fn new(device: &GfxDevice, size: WindowSize, target_format: TextureFormat) -> Self {
        let textures = Textures::new(device, size);
        let sampler = Self::create_sampler(device);
        let depth_sampler = Self::create_depth_sampler(device);

        let gbuf_pipeline = GBufferPipeline::new(device, target_format);
        let bind_group = Self::create_bind_group(
            device,
            &textures,
            &sampler,
            &depth_sampler,
            &gbuf_pipeline.bind_group_layout,
        );

        Self {
            textures,
            sampler,
            depth_sampler,
            gbuf_pipeline,
            bind_group,
        }
    }

    pub fn render(&self, encoder: &mut CommandEncoder, target: &TextureView) {
        let attachment = RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
        };
        let attachments = [attachment];
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments: &attachments,
            ..Default::default()
        };

        let mut pass = encoder.begin_render_pass(&render_pass_descriptor);
        pass.set_pipeline(&self.gbuf_pipeline.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn get_views(&self) -> [&TextureView; 3] {
        [
            &self.textures.position_view,
            &self.textures.albedo_view,
            &self.textures.depth_view,
        ]
    }

    pub fn resize(&mut self, device: &GfxDevice, size: WindowSize) {
        self.textures = Textures::new(device, size);
        self.bind_group = Self::create_bind_group(
            device,
            &self.textures,
            &self.sampler,
            &self.depth_sampler,
            &self.gbuf_pipeline.bind_group_layout,
        );
    }

    fn create_sampler(device: &GfxDevice) -> Sampler {
        device.device.create_sampler(&Default::default())
    }

    fn create_depth_sampler(device: &GfxDevice) -> Sampler {
        device.device.create_sampler(&SamplerDescriptor {
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        })
    }

    fn create_bind_group(
        device: &GfxDevice,
        textures: &Textures,
        sampler: &Sampler,
        depth_sampler: &Sampler,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let position_binding = BindingResource::TextureView(&textures.position_view);
        let albedo_binding = BindingResource::TextureView(&textures.albedo_view);
        let depth_binding = BindingResource::TextureView(&textures.depth_view);
        let sampler_binding = BindingResource::Sampler(sampler);
        let depth_sampler_binding = BindingResource::Sampler(depth_sampler);

        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: position_binding,
            },
            BindGroupEntry {
                binding: 1,
                resource: albedo_binding,
            },
            BindGroupEntry {
                binding: 2,
                resource: depth_binding,
            },
            BindGroupEntry {
                binding: 3,
                resource: sampler_binding,
            },
            BindGroupEntry {
                binding: 4,
                resource: depth_sampler_binding,
            },
        ];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }
}

struct Textures {
    pub position_view: TextureView,
    pub albedo_view: TextureView,
    pub depth_view: TextureView,
}

impl Textures {
    pub fn new(device: &GfxDevice, size: WindowSize) -> Self {
        let position = Self::create_position_texture(device, size);
        let albedo = Self::create_albedo_texture(device, size);
        let depth = Self::create_depth_texture(device, size);

        let position_view = position.create_view(&Default::default());
        let albedo_view = albedo.create_view(&Default::default());
        let depth_view = depth.create_view(&Default::default());

        Self {
            position_view,
            albedo_view,
            depth_view,
        }
    }

    fn create_position_texture(device: &GfxDevice, size: WindowSize) -> Texture {
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let descriptor = TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: GBuffer::POSITION_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        };

        device.device.create_texture(&descriptor)
    }

    fn create_albedo_texture(device: &GfxDevice, size: WindowSize) -> Texture {
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let descriptor = TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: GBuffer::ALBEDO_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        };

        device.device.create_texture(&descriptor)
    }

    fn create_depth_texture(device: &GfxDevice, size: WindowSize) -> Texture {
        let size = Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        };

        let descriptor = TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: GBuffer::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        };

        device.device.create_texture(&descriptor)
    }
}
