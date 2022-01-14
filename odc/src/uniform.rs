use crate::{GfxDevice, RenderInfo};
use std::mem;
use std::num::NonZeroU64;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferSize, BufferUsages,
    ShaderStages,
};

pub struct Uniform {
    pub buffer: Buffer,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Uniform {
    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::UNIFORM;
        let buffer = device.create_gpu_buffer(Self::buffer_size().get(), usage);
        let layout = Self::create_bind_group_layout(device);
        let bind_group = Self::create_bind_group(device, &buffer, &layout);

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    fn buffer_size() -> BufferSize {
        NonZeroU64::new(mem::size_of::<RenderInfo>() as _).expect("Zero sized uniform")
    }

    fn create_bind_group_layout(device: &GfxDevice) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(Self::buffer_size()),
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

    fn create_bind_group(
        device: &GfxDevice,
        buffer: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }
}
