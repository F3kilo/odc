use glam::{Mat4, Quat, Vec3};
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
    renderer.write_mesh(&rectangle_mesh(), 3, 3);

    let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
    let scale = Vec3::new(0.4, 0.4, 0.4);
    let left = Vec3::new(-0.5, 0.0, 0.0);
    let right = Vec3::new(0.5, 0.0, 0.0);
    let left_transform =
        Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, left).to_cols_array_2d();
    let right_transform =
        Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, right).to_cols_array_2d();

    let left_instance = InstanceInfo {
        transform: left_transform,
    };
    let right_instance = InstanceInfo {
        transform: right_transform,
    };

    renderer.write_instances(&[left_instance, right_instance], 0);

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

                let draw_triangle = StaticMesh {
                    indices: 0..3,
                    base_vertex: 0,
                    instances: 0..1,
                };

                let draw_rectangle = StaticMesh {
                    indices: 3..9,
                    base_vertex: 3,
                    instances: 1..2,
                };
                renderer.render(&info, [draw_triangle, draw_rectangle].iter());
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

fn rectangle_mesh() -> Mesh {
    Mesh {
        vertices: RECTANGLE_VERTICES.to_vec(),
        indices: RECTANGLE_INDICES.to_vec(),
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

const RECTANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 1.0, 1.0],
    },
];

const RECTANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];
