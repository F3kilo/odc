mod common;

use glam::Mat4;
use odc::{InstanceInfo, OdcCore, RenderInfo, StaticMesh, WindowSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};



fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);

    let mut renderer = OdcCore::new(&window, size);
    let (vertex_data, index_data) = common::triangle_mesh();
    renderer.write_vertices(vertex_data, 0);
    renderer.write_indices(index_data, 0);

    let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
    let instance = InstanceInfo {
        transform: ident_transform,
    };
    renderer.write_instances(&[instance], 0);

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
                let info = RenderInfo {
                    world: ident_transform,
                    view_proj: ident_transform,
                };

                let draw = StaticMesh {
                    indices: 0..3,
                    base_vertex: 0,
                    instances: 0..1,
                };
                renderer.render(&info, [draw].iter());
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
