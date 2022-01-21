use crate::GfxDevice;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ShaderStages,
};

pub struct Instances {
    pub buffer: Buffer,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Instances {
    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::STORAGE;
        let buffer = device.create_gpu_buffer(Self::instances_size(), usage);
        let layout = Self::create_layout(device);
        let bind_group = Self::create_bind_group(device, &layout, &buffer);

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    pub const INSTANCE_BUFFER_SIZE: u64 = 2u64.pow(16);

    fn instances_size() -> u64 {
        Self::INSTANCE_BUFFER_SIZE
    }

    fn create_layout(device: &GfxDevice) -> BindGroupLayout {
        let storage_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[storage_entry],
        };
        device.device.create_bind_group_layout(&descriptor)
    }

    pub fn create_bind_group(
        device: &GfxDevice,
        layout: &BindGroupLayout,
        storage: &Buffer,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: storage.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }
}
