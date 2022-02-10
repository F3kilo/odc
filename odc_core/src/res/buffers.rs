pub struct Buffers {
    pub index: Buffer,
    pub vertex: Buffer,
    pub instance: Buffer,
    pub uniform: Buffer,
}

pub struct Buffer {
    pub handle: wgpu::Buffer,
    pub info: BufferInfo,
}

pub struct BufferInfo {
    pub size: u64,
    pub usage: wgpu::BufferUsages,
}
