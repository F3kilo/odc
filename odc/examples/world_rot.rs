mod common;

use crate::common::{InstanceInfo, RenderInfo};
use glam::{Mat4, Vec3};
use odc::config::{WindowConfig};
use odc::{Draws, Odc, StaticMesh, Transform, WindowSize};
use std::f32::consts::PI;
use std::time::Instant;
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

    let rotation = Rotation::default();

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
                let view_proj = Mat4::IDENTITY.to_cols_array_2d();
                let info = RenderInfo {
                    world: rotation.transform(),
                    view_proj,
                };
                renderer.write_buffer(&3, &[info], 0);

                let instance = InstanceInfo {
                    transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
                };
                renderer.write_buffer(&1, &[instance], 0);

                let draw = StaticMesh {
                    indices: 0..3,
                    base_vertex: 0,
                    instances: 0..1,
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

struct Rotation {
    start: Instant,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Rotation {
    pub fn transform(&self) -> Transform {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let angle = (2.0 * PI * elapsed) % (2.0 * PI);
        let rotation = Mat4::from_rotation_z(angle);
        let scale = Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));
        (rotation * scale).to_cols_array_2d()
    }
}
