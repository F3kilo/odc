use crate::GfxDevice;
use wgpu::{Buffer, BufferUsages, RenderPass};

pub struct Instances {
    buffer: Buffer,
}

impl Instances {
    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::VERTEX;
        let buffer = device.create_gpu_buffer(Self::instances_size(), usage);

        Self { buffer }
    }

    pub fn write(&self, device: &GfxDevice, data: &[u8], offset: u64) {
        device.queue.write_buffer(&self.buffer, offset, data)
    }

    pub const INSTANCE_BUFFER_SIZE: u64 = 2u64.pow(16);

    pub fn bind<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_vertex_buffer(1, self.buffer.slice(..));
    }

    fn instances_size() -> u64 {
        Self::INSTANCE_BUFFER_SIZE
    }
}
