mod bind;
mod buffers;
mod samplers;
mod textures;

pub use bind::{
    BindGroupFactory, BindGroupInfo, BindGroups, Binding, SamplerBindingInfo, TextureBindingInfo,
    UniformBindingInfo,
};
pub use buffers::{Buffer, BufferInfo, BufferType, Buffers};
pub use samplers::{Sampler, SamplerInfo};
use std::collections::HashMap;
pub use textures::{Texture, TextureInfo};

pub struct Resources {
    pub buffers: Buffers,
    pub textures: Vec<Texture>,
    pub samplers: Vec<Sampler>,
    pub stock: Stock,
}

impl Resources {
    pub fn insert_stock_buffer(
        &mut self,
        device: &wgpu::Device,
        typ: BufferType,
        name: String,
        size: Option<u64>,
    ) {
        let mut info = self.buffers.get(typ).info.clone();
        info.size = size.unwrap_or(info.size);
        let factory = ResourceFactory::new(device);
        let new_buffer = factory.create_buffer(info);
        self.stock.insert_buffer(name, typ, new_buffer)
    }

    pub fn remove_stock_buffer(&mut self, name: &str) {
        self.stock.remove_buffer(name);
    }

    pub fn swap_stock_buffer(&mut self, name: &str) {
        let (name, (typ, buffer)) = self.stock.remove_buffer(&name);
        let old_buffer = self.buffers.replace(typ, buffer);
        self.stock.insert_buffer(name, typ, old_buffer);
    }

    pub fn insert_stock_texture(
        &mut self,
        device: &wgpu::Device,
        id: usize,
        name: String,
        size: Option<wgpu::Extent3d>,
    ) {
        let mut info = self.textures[id].info.clone();
        info.size = size.unwrap_or(info.size);
        let factory = ResourceFactory::new(device);
        let new_texture = factory.create_texture(info);
        self.stock.insert_texture(name, id, new_texture);
    }

    pub fn swap_stock_texture(&mut self, name: &str) {
        let (name, (id, texture)) = self.stock.remove_texture(&name);
        let replaced = std::mem::replace(&mut self.textures[id], texture);
        self.stock.insert_texture(name, id, replaced);
    }

    pub fn remove_stock_texture(&mut self, name: &str) {
        self.stock.remove_texture(name);
    }
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
            address_mode_u: info.u_address,
            address_mode_v: info.v_address,
            address_mode_w: info.w_address,
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

#[derive(Default)]
pub struct Stock {
    buffers: HashMap<String, (BufferType, Buffer)>,
    textures: HashMap<String, (usize, Texture)>,
}

impl Stock {
    pub fn buffer_type(&self, name: &str) -> BufferType {
        self.buffers[name].0
    }

    pub fn insert_buffer(&mut self, name: String, typ: BufferType, buffer: Buffer) {
        self.buffers.insert(name, (typ, buffer));
    }

    pub fn swap_buffer(&mut self, name: &str, buffer: Buffer) -> (BufferType, Buffer) {
        let mut entry = self.buffers.get_mut(name).unwrap();
        let replaced = std::mem::replace(&mut entry.1, buffer);
        (entry.0, replaced)
    }

    pub fn remove_buffer(&mut self, name: &str) -> (String, (BufferType, Buffer)) {
        self.buffers.remove_entry(name).unwrap()
    }

    pub fn texture_id(&self, name: &str) -> usize {
        self.textures[name].0
    }

    pub fn insert_texture(&mut self, name: String, id: usize, texture: Texture) {
        self.textures.insert(name, (id, texture));
    }

    pub fn swap_texture(&mut self, name: &str, texture: Texture) -> (usize, Texture) {
        let mut entry = self.textures.get_mut(name).unwrap();
        let replaced = std::mem::replace(&mut entry.1, texture);
        (entry.0, replaced)
    }

    pub fn remove_texture(&mut self, name: &str) -> (String, (usize, Texture)) {
        self.textures.remove_entry(name).unwrap()
    }
}
