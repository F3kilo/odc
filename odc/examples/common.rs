#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use fps_counter::FPSCounter;
use odc::material::{InputAttribute, InputInfo, MaterialInfo, ShaderInfo};
use odc::{DrawData, Odc, WindowSize};
use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

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
    fn init(&mut self, renderer: &mut Odc);
    fn update(&mut self, renderer: &Odc);
    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)>;
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
                StartCause::Init => ex.init(&mut renderer),
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
                let to_draw = ex.draw_info();
                let draw_map =
                    HashMap::from_iter(to_draw.iter().map(|(id, d)| (*id, d.as_slice())));
                renderer.render(&draw_map);
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

pub struct MaterialData {
    pub vertex_attributes: Vec<InputAttribute>,
    pub vertex_stride: u64,
    pub instance_attributes: Vec<InputAttribute>,
    pub instance_stride: u64,
    pub shader_source: String,
    pub vs_entry: String,
    pub fs_entry: String,
    pub uniform_location: Range<u64>,
}

impl MaterialData {
    pub fn as_info(&self) -> MaterialInfo {
        MaterialInfo {
            vertex: InputInfo {
                attributes: &self.vertex_attributes,
                stride: self.vertex_stride,
            },
            instance: InputInfo {
                attributes: &self.instance_attributes,
                stride: self.instance_stride,
            },
            shader: ShaderInfo {
                source: &self.shader_source,
                vs_entry: &self.vs_entry,
                fs_entry: &self.fs_entry,
            },
            uniform_location: self.uniform_location.clone(),
        }
    }
}

pub fn color_mesh_material_data() -> MaterialData {
    let vertex_attributes =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4].to_vec();
    let instance_attributes =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4]
            .to_vec();

    let shader_source = include_str!("shaders/color_mesh.wgsl").to_string();
    let vs_entry = "vs_main".to_string();
    let fs_entry = "fs_main".to_string();
    
    let mat4_size = 16 * 4;
    let uniform_location = 0..(mat4_size * 2);

    MaterialData {
        vertex_attributes,
        vertex_stride: 12 * 4,
        instance_attributes,
        instance_stride: mat4_size,
        shader_source,
        vs_entry,
        fs_entry,
        uniform_location
    }
}

pub fn blue_mesh_material_data() -> MaterialData {
    let vertex_attributes =
        wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4].to_vec();
    let instance_attributes =
        wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4]
            .to_vec();

    let shader_source = include_str!("shaders/blue_mesh.wgsl").to_string();
    let vs_entry = "vs_main".to_string();
    let fs_entry = "fs_main".to_string();
    
    let mat4_size = 16 * 4;
    let uniform_location = 0..(mat4_size * 2);

    MaterialData {
        vertex_attributes,
        vertex_stride: 12 * 4,
        instance_attributes,
        instance_stride: mat4_size,
        shader_source,
        vs_entry,
        fs_entry,
        uniform_location
    }
}

fn main() {}
