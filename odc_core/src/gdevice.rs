use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, Limits, Queue, RequestAdapterOptions, Surface,
};

pub struct GfxDevice {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GfxDevice {
    pub fn new(instance: &Instance, surface: Option<&Surface>) -> Self {
        let adapter = Self::request_adapter(instance, surface);
        let (device, queue) = Self::request_device(&adapter);
        Self {
            adapter,
            device,
            queue,
        }
    }

    fn request_adapter(instance: &Instance, surface: Option<&Surface>) -> Adapter {
        let options = RequestAdapterOptions {
            compatible_surface: surface,
            ..Default::default()
        };
        let adapter_fut = instance.request_adapter(&options);
        pollster::block_on(adapter_fut).unwrap()
    }

    fn request_device(adapter: &Adapter) -> (Device, Queue) {
        let limits = Limits::downlevel_defaults().using_resolution(adapter.limits());
        let descriptor = DeviceDescriptor {
            limits,
            ..Default::default()
        };
        let device_fut = adapter.request_device(&descriptor, None);
        pollster::block_on(device_fut).unwrap()
    }
}
