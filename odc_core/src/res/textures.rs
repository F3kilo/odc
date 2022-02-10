pub struct Texture {
    pub handle: wgpu::Texture,
    pub info: TextureInfo,
}

impl Texture {
    pub fn create_view(&self) -> wgpu::TextureView {
        self.handle.create_view(&Default::default())
    }
}

pub struct TextureInfo {
    pub format: wgpu::TextureFormat,
    pub size: wgpu::Extent3d,
    pub usages: wgpu::TextureUsages,
}