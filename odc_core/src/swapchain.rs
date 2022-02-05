use crate::mdl;
use wgpu::{PresentMode, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};

pub struct Swapchain {
    pub surface: Surface,
    pub format: TextureFormat,
}

impl Swapchain {
    pub fn new(surface: Surface, adapter: &wgpu::Adapter) -> Self {
        let format = surface
            .get_preferred_format(adapter)
            .expect("can't find suit surface format");

        Self { surface, format }
    }

    pub fn resize(&self, device: &wgpu::Device, size: mdl::Size2d) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width: size.x as _,
            height: size.y as _,
            present_mode: PresentMode::Mailbox,
        };

        self.surface.configure(&device, &config);
    }
}
