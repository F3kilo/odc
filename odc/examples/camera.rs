mod common;

use crate::common::InstanceInfo;
use glam::Mat4;
use odc::config::WindowConfig;
use odc::{Draws, Odc, RenderInfo, StaticMesh, WindowSize};
use std::f32::consts::PI;
use std::time::Instant;
use vp_cam::{Camera, CameraBuilder, Vec3};
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
    renderer.write_buffer(&1, index_data, 0);

    let rotation = CameraMovement::default();
    let mut camera = create_camera();

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
                let world = Mat4::IDENTITY.to_cols_array_2d();
                camera.set_position(rotation.cam_position());
                let view_proj = camera.view_proj_transform();

                let info = RenderInfo { world, view_proj };
                let instance = InstanceInfo {
                    transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
                };
                renderer.write_instances(&[instance], 0);

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

fn create_camera() -> Camera {
    let pos = [0.0, 0.0, -CameraMovement::RADIUS];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
}

struct CameraMovement {
    start: Instant,
}

impl Default for CameraMovement {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl CameraMovement {
    pub const RADIUS: f32 = 4.0;

    pub fn cam_position(&self) -> Vec3 {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let angle = (2.0 * PI * elapsed) % (2.0 * PI);

        let x = Self::RADIUS * angle.sin();
        let z = Self::RADIUS * -angle.cos();
        [x, 0.0, z]
    }
}
