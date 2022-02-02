use crate::gdevice::GfxDevice;
use model as st;
use model::Size2d;
use raw_window_handle::HasRawWindowHandle;
use res::Resources;
use bind::BindGroups;
use pipelines::Pipelines;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{Backends, Instance};

mod gdevice;
pub mod model;
mod res;
mod pipelines;
mod bind;
mod swapchain;

pub struct OdcCore {
    device: GfxDevice,
    swapchain: Swapchain,
    resources: Resources,
    bind_groups: BindGroups,
    pipelines: Pipelines,
}

impl OdcCore {
    pub fn with_window_support(model: st::RenderModel, window: &impl HasRawWindowHandle) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let swapchain = Swapchain::new(&device, surface);
        let resources = Resources::new(&device.device, &model);
        let bind_groups = BindGroups::new(&device.device, &model, &resources);
        let pipelines = Pipelines::new(&device.device, &model, &bind_groups, swapchain.format);

        Self {
            device,
            swapchain,
            resources,
            bind_groups,
            pipelines,
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
