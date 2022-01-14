use crate::{GfxDevice, Vertex};
use std::mem;
use wgpu::{Buffer, BufferUsages, IndexFormat, RenderPass};

pub struct MeshBuffers {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl MeshBuffers {
    pub const VERTEX_BUFFER_SIZE: u64 = 2u64.pow(24);
    pub const INDEX_BUFFER_SIZE: u64 = 2u64.pow(22);

    pub fn new(device: &GfxDevice) -> Self {
        let usages = BufferUsages::COPY_DST | BufferUsages::VERTEX;
        let vertex_buffer = device.create_gpu_buffer(Self::VERTEX_BUFFER_SIZE, usages);

        let usages = BufferUsages::COPY_DST | BufferUsages::INDEX;
        let index_buffer = device.create_gpu_buffer(Self::INDEX_BUFFER_SIZE, usages);

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn write_vertices(&self, device: &GfxDevice, vertices: &[Vertex], offset: u64) {
        let data = bytemuck::cast_slice(vertices);
        let offset = offset * Vertex::size() as u64;
        device.queue.write_buffer(&self.vertex_buffer, offset, data);
    }

    pub fn write_indices(&self, device: &GfxDevice, indices: &[u32], offset: u64) {
        let data = bytemuck::cast_slice(indices);
        let offset = offset * mem::size_of::<u32>() as u64;
        device.queue.write_buffer(&self.index_buffer, offset, data);
    }

    pub fn bind<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    }
}
