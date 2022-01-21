mod common;

use crate::common::InstanceInfo;
use glam::{Vec3, Mat4};
use odc::config::WindowConfig;
use odc::{Draws, Odc, RenderInfo, StaticMesh, WindowSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);

    let window_config = WindowConfig {
        handle: &window,
        size,
    };
    let config = common::color_mesh_renderer_config(window_config);
    let mut renderer = Odc::new(&config);
    let (vertex_data, index_data) = common::triangle_mesh();
    renderer.write_buffer(&0, vertex_data, 0);
    renderer.write_buffer(&2, index_data, 0);

    let ident_transform = Mat4::IDENTITY.to_cols_array_2d();

    let scale = Vec3::new(0.9, 0.5, 0.5);
    let instance = InstanceInfo {
        transform: Mat4::from_scale(scale).to_cols_array_2d(),
    };
    renderer.write_buffer(&1, &[instance], 0);

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
                let draws = Draws {
                    static_mesh: &[draw],
                };
                renderer.render(&info, draws);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
