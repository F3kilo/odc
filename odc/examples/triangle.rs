use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use odc::{TriangleRenderer, WindowSize};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);
    let mut renderer = TriangleRenderer::new(&window, size);
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = WindowSize(size.width, size.height);
                renderer.resize(size);
            },
            Event::RedrawRequested(_) => {
                renderer.render_triangle();
            }
            _ => {}
        }
    });
}