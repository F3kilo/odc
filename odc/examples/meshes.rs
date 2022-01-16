mod common;

use crate::common::InstanceInfo;
use glam::{Mat4, Quat, Vec3};
use odc::{OdcCore, RenderInfo, StaticMesh, WindowSize};
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

    let vertex_offset = vertex_data.len();
    let index_offset = index_data.len();
    let (vertex_data, index_data) = common::rectangle_mesh();
    renderer.write_vertices(vertex_data, vertex_offset as _);
    renderer.write_indices(index_data, index_offset as _);

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

    let instances = [left_instance, right_instance];
    renderer.write_instances(bytemuck::cast_slice(&instances), 0);

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
