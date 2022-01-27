#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use fps_counter::FPSCounter;
use odc::Transform;
use odc::{Draws, Odc, RenderInfo, StaticMesh, WindowSize};
use std::mem;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Copy, Clone)]
pub struct InstanceInfo {
    pub transform: Transform,
}

impl InstanceInfo {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }
}

unsafe impl Zeroable for InstanceInfo {}
unsafe impl Pod for InstanceInfo {}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }

    pub const fn position_offset() -> usize {
        0
    }

    pub const fn color_offset() -> usize {
        mem::size_of::<[f32; 4]>()
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

pub fn triangle_mesh() -> (&'static [Vertex], &'static [u32]) {
    (&TRIANGLE_VERTICES, &TRIANGLE_INDICES)
}

pub const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        normal: [-1.0, -1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0, 1.0],
        normal: [0.0, 1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        normal: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];

pub fn rectangle_mesh() -> (&'static [Vertex], &'static [u32]) {
    (&RECTANGLE_VERTICES, &RECTANGLE_INDICES)
}

pub const RECTANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        normal: [-1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        normal: [-1.0, 1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0, 1.0],
        normal: [1.0, 1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        normal: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 1.0, 1.0],
    },
];

pub const RECTANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub trait Example {
    fn init(&mut self, renderer: &Odc);
    fn update(&mut self, renderer: &Odc);
    fn draw_info(&self) -> (RenderInfo, Vec<StaticMesh>);
}

pub fn run_example(mut ex: impl Example + 'static) -> ! {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let size = window.inner_size();
    let size = WindowSize(size.width, size.height);

    let mut renderer = Odc::new(&window, size);
    let mut fps_counter = FPSCounter::new();

    event_loop.run(move |event, _, flow| {
        *flow = ControlFlow::Poll;
        match event {
            Event::NewEvents(cause) => match cause {
                StartCause::Init => ex.init(&renderer),
                StartCause::Poll => ex.update(&renderer),
                _ => {}
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let size = WindowSize(size.width, size.height);
                renderer.resize(size);
            }
            Event::MainEventsCleared => {
                let (info, static_mesh) = ex.draw_info();
                let draws = Draws {
                    static_mesh: &static_mesh[..],
                };
                renderer.render(&info, draws);
                let fps = fps_counter.tick();
                window.set_title(&format!("FPS: {}", fps));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {}
