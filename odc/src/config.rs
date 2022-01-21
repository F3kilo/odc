use crate::WindowSize;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;

pub struct Config<Window: HasRawWindowHandle> {
    pub window: Option<WindowConfig<Window>>,
    pub device: DeviceConfig,
    pub resources: HashMap<u64, ResourceConfig>,
}

pub struct WindowConfig<Window: HasRawWindowHandle> {
    pub handle: Window,
    pub size: WindowSize,
}

pub struct DeviceConfig {
    pub name: Option<String>,
}

pub enum ResourceConfig {
    VertexBuffer(u64),
    IndexBuffer(u64),
    // UniformBuffer(u64),
}
