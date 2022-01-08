use glam::Mat4;
use odc::{InstanceInfo, RenderInfo, TriangleRenderer, WindowSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);
    let mut renderer = TriangleRenderer::new(&window, size);
    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = WindowSize(size.width, size.height);
                renderer.resize(size);
            }
            Event::MainEventsCleared => {
                let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
                let info = RenderInfo {
                    world: ident_transform,
                    view_proj: ident_transform,
                };

                let instance = InstanceInfo {
                    transform: ident_transform,
                };
                renderer.render_triangle(&info, &[instance]);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
