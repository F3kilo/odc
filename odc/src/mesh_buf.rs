use std::mem;
use wgpu::{Buffer, BufferUsages, IndexFormat, RenderPass};
use crate::{GfxDevice, Vertex};

pub struct MeshBuffers {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl MeshBuffers {
    pub fn new(device: &GfxDevice, vertex_buffer_size: u64, index_buffer_size: u64) -> Self {
        let vertex_buffer = device.create_gpu_buffer(
            vertex_buffer_size,
            BufferUsages::COPY_DST | BufferUsages::VERTEX,
        );
        let index_buffer = device.create_gpu_buffer(
            index_buffer_size,
            BufferUsages::COPY_DST | BufferUsages::INDEX,
        );

        Self {
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn write_vertices(&self, device: &GfxDevice, vertices: &[Vertex], offset: u64) {
        let data = bytemuck::cast_slice(vertices);
        let offset = offset * Vertex::size() as u64;
        device.write_buffer(&self.vertex_buffer, offset, data);
    }

    pub fn write_indices(&self, device: &GfxDevice, indices: &[u32], offset: u64) {
        let data = bytemuck::cast_slice(indices);
        let offset = offset * mem::size_of::<u32>() as u64;
        device.write_buffer(&self.index_buffer, offset, data);
    }

    pub fn bind<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    }
}

