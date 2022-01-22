mod common;

use crate::common::{InstanceInfo, RenderInfo};
use bytemuck::Zeroable;
use glam::Mat4;
use odc::config::{WindowConfig};
use odc::{Draws, Odc, StaticMesh, WindowSize};
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
                renderer.write_buffer(&3, &[info], 0);

                let instances = get_instances();
                renderer.write_buffer(&1, &[instances], 0);

                let draw = StaticMesh {
                    indices: 0..3,
                    base_vertex: 0,
                    instances: 0..256,
                };
                let draws = Draws {
                    static_mesh: &[draw],
                };
                renderer.render(draws);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn get_instances() -> [InstanceInfo; 256] {
    let mut instances = [InstanceInfo::zeroed(); 256];

    for i in 0..16 {
        for j in 0..16 {
            instances[i * 16 + j] = create_instance(i, j);
        }
    }

    instances
}

fn create_instance(x: usize, y: usize) -> InstanceInfo {
    let step = 2.0 / 16.0;
    let x = (x as f32 - 7.5) * step;
    let y = (y as f32 - 7.5) * step;
    let pos = glam::vec3(x, y, 0.0);
    let size = step / 2.5;
    let scale = glam::vec3(size, size, size);
    let transform = glam::Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, pos)
        .to_cols_array_2d();
    InstanceInfo { transform }
}
