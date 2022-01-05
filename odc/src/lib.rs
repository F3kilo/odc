use raw_window_handle::HasRawWindowHandle;

pub struct TriangleRenderer {}

impl TriangleRenderer {
    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        todo!()
    }

    pub fn render_triangle(&self) {
        todo!()
    }

    pub fn resize(&mut self, size: WindowSize) {
        todo!()
    }
}

pub struct WindowSize(pub u32, pub u32);
