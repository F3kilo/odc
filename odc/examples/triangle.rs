use glam::Mat4;
use odc::{InstanceInfo, Mesh, RenderInfo, StaticMesh, TriangleRenderer, Vertex, WindowSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);

    let mut renderer = TriangleRenderer::new(&window, size);
    renderer.write_mesh(&triangle_mesh(), 0, 0);
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
                renderer.render_triangle(&info, [draw].iter());
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn triangle_mesh() -> Mesh {
    Mesh {
        vertices: TRIANGLE_VERTICES.to_vec(),
        indices: TRIANGLE_INDICES.to_vec(),
    }
}

const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];
