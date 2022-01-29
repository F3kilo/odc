use crate::gdevice::GfxDevice;
use crate::structure::{RenderStructure, Size2d};
use raw_window_handle::HasRawWindowHandle;
use render::RenderData;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{Backends, Instance};

mod gdevice;
mod render;
mod structure;
mod swapchain;

pub struct OdcCore {
    device: GfxDevice,
    swapchain: Swapchain,
    data: RenderData,
}

impl OdcCore {
    pub fn with_window(structure: &RenderStructure, window: WindowInfo) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window.handle) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let swapchain = Swapchain::new(&device, surface);

        let data = RenderData::from_structure(&device, structure);

        Self {
            device,
            swapchain,
            data,
        }
    }
}

pub struct WindowInfo<'a> {
    pub name: String,
    pub handle: &'a dyn HasRawWindowHandle,
    pub size: Size2d,
}

pub struct DrawData {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}
