pub struct Buffer {
    pub handle: wgpu::Buffer,
    pub info: BufferInfo,
}

pub struct BufferInfo {
    pub name: String,
    pub size: u64,
    pub usage: wgpu::BufferUsages,
}
