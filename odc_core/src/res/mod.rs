mod bind;
mod buffers;
mod samplers;
mod textures;

pub use bind::{
    BindGroupFactory, BindGroupInfo, BindGroups, Binding, SamplerBindingInfo, TextureBindingInfo,
    UniformBindingInfo,
};
pub use buffers::{Buffer, BufferInfo, Buffers};
pub use samplers::{Sampler, SamplerInfo};
pub use textures::{Texture, TextureInfo};

pub struct Resources {
    pub buffers: Buffers,
    pub textures: Vec<Texture>,
    pub samplers: Vec<Sampler>,
}

pub struct ResourceFactory<'a> {
    device: &'a wgpu::Device,
}

impl<'a> ResourceFactory<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self { device }
    }

    pub fn create_buffer(&self, info: BufferInfo) -> Buffer {
        let handle = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: info.size,
            usage: info.usage,
            mapped_at_creation: false,
        });

        Buffer { handle, info }
    }

    pub fn create_texture(&self, info: TextureInfo) -> Texture {
        let handle = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: info.size,
            format: info.format,
            usage: info.usages,
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: info.mip_levels,
            sample_count: info.sample_count,
        });

        Texture { handle, info }
    }

    pub fn create_sampler(&self, info: SamplerInfo) -> Sampler {
        let handle = self.device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: info.mode,
            mag_filter: info.mode,
            mipmap_filter: info.mode,
            compare: info.compare,
            anisotropy_clamp: info.anisotropy,
            ..Default::default()
        });

        Sampler { handle, info }
    }
}
