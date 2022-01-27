use crate::GfxDevice;
use std::num::NonZeroU64;
use std::ops::Range;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType,
    BufferUsages, ShaderStages,
};

pub struct Uniform {
    pub buffer: Buffer,
    pub layout: BindGroupLayout,
}

impl Uniform {
    pub const BUFFER_SIZE: u64 = 2u64.pow(16);

    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::UNIFORM;
        let buffer = device.create_gpu_buffer(Self::BUFFER_SIZE, usage);
        let layout = Self::create_bind_group_layout(device);

        Self { buffer, layout }
    }

    pub fn get_bind_group_layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    pub fn write(&self, device: &GfxDevice, data: &[u8], offset: u64) {
        device.queue.write_buffer(&self.buffer, offset, data)
    }

    fn create_bind_group_layout(device: &GfxDevice) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[uniform_entry],
        };
        device.device.create_bind_group_layout(&descriptor)
    }

    pub fn create_bind_group(&self, device: &GfxDevice, range: &Range<u64>) -> BindGroup {
        let size =
            NonZeroU64::new(range.end - range.start).expect("unexpected zero uniform binding size");
        let binding = BufferBinding {
            buffer: &self.buffer,
            offset: range.start,
            size: Some(size),
        };

        let entries = [BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(binding),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }
}
