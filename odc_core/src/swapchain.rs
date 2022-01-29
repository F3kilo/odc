use crate::{GfxDevice, Size2d};
use wgpu::{PresentMode, Surface, SurfaceConfiguration, TextureFormat, TextureUsages};

pub struct Swapchain {
    pub surface: Surface,
    pub format: TextureFormat,
}

impl Swapchain {
    pub fn new(device: &GfxDevice, surface: Surface) -> Self {
        let format = surface
            .get_preferred_format(&device.adapter)
            .expect("can't find suit surface format");
        Self { surface, format }
    }

    pub fn resize(&self, device: &GfxDevice, size: Size2d) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            width: size.x as _,
            height: size.y as _,
            present_mode: PresentMode::Mailbox,
        };

        self.surface.configure(&device.device, &config);
    }
}
