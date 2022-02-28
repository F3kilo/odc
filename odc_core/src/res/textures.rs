pub struct Texture {
    pub handle: wgpu::Texture,
    pub info: TextureInfo,
}

impl Texture {
    pub fn create_view(&self, dimension: Option<wgpu::TextureViewDimension>) -> wgpu::TextureView {
        self.handle.create_view(&wgpu::TextureViewDescriptor {
            dimension,
            ..Default::default()
        })
    }
}

pub struct TextureInfo {
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
    pub usages: wgpu::TextureUsages,
    pub mip_levels: u32,
    pub sample_count: u32,
}
