pub struct Buffers {
    index: Buffer,
    vertex: Buffer,
    instance: Buffer,
    uniform: Buffer,
}

impl Buffers {
    pub fn new(index: Buffer, vertex: Buffer, instance: Buffer, uniform: Buffer) -> Self {
        Self {
            index,
            vertex,
            instance,
            uniform,
        }
    }

    pub fn get(&self, typ: BufferType) -> &Buffer {
        match typ {
            BufferType::Index => &self.index,
            BufferType::Vertex => &self.vertex,
            BufferType::Instance => &self.instance,
            BufferType::Uniform => &self.uniform,
        }
    }

    pub fn bind_buffers<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_index_buffer(self.index.handle.slice(..), wgpu::IndexFormat::Uint32);
        pass.set_vertex_buffer(0, self.vertex.handle.slice(..));
        pass.set_vertex_buffer(1, self.instance.handle.slice(..));
    }
}

pub struct Buffer {
    pub handle: wgpu::Buffer,
    pub info: BufferInfo,
}

#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub size: u64,
    pub usage: wgpu::BufferUsages,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BufferType {
    Index,
    Vertex,
    Instance,
    Uniform,
}
