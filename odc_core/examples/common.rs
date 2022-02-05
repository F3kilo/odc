#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use fps_counter::FPSCounter;
use gltf::{buffer, Accessor, Semantic};
use odc_core::model::{RenderModel, Size2d};
use odc_core::{DrawData, OdcCore, WindowInfo};
use std::mem;
use std::ops::Range;
use std::path::Path;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Copy, Clone)]
pub struct ColorVertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl ColorVertex {
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

unsafe impl Zeroable for ColorVertex {}
unsafe impl Pod for ColorVertex {}

pub fn triangle_mesh() -> (&'static [ColorVertex], &'static [u32]) {
    (&TRIANGLE_VERTICES, &TRIANGLE_INDICES)
}

pub const TRIANGLE_VERTICES: [ColorVertex; 3] = [
    ColorVertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [0.0, 1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];

pub fn rectangle_mesh() -> (&'static [ColorVertex], &'static [u32]) {
    (&RECTANGLE_VERTICES, &RECTANGLE_INDICES)
}

pub const RECTANGLE_VERTICES: [ColorVertex; 4] = [
    ColorVertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [1.0, 1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    ColorVertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 1.0, 1.0],
    },
];

pub const RECTANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub trait Example {
    fn render_model() -> RenderModel;
    fn init(&mut self, renderer: &OdcCore);
    fn update(&mut self, renderer: &OdcCore);
    fn draw_info(&self) -> (Vec<DrawData>, Vec<Range<usize>>);
}

pub fn run_example<E: Example + 'static>(mut ex: E) -> ! {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let window_info = WindowInfo {
        handle: &window,
        size: Size2d {
            x: window.inner_size().width as _,
            y: window.inner_size().height as _,
        }
    };
    let mut renderer = OdcCore::with_window_support(E::render_model(), &window);
    let mut fps_counter = FPSCounter::new();
    unsafe {renderer.add_window("color", window_info) };
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
                let size = Size2d {
                    x: size.width as _,
                    y: size.height as _,
                };
                renderer.resize_window("color", size);
            }
            Event::MainEventsCleared => {
                let (data, ranges) = ex.draw_info();
                renderer.draw(&data, ranges.into_iter());
                let fps = fps_counter.tick();
                window.set_title(&format!("FPS: {}", fps));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                renderer.remove_window("color");
                *flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

pub type VertexData = Vec<u8>;
pub type IndexData = Vec<u8>;

pub fn load_gltf_mesh<P: AsRef<Path>>(path: P) -> (VertexData, IndexData) {
    let (doc, buffers, _) = gltf::import(path).unwrap();
    let mesh = doc.meshes().next().unwrap();
    let primitive = mesh.primitives().next().unwrap();

    let mut mesh_data = MeshData::default();
    let index_accessor = primitive.indices().unwrap();
    mesh_data.index_data_from_accessor(index_accessor, &buffers);
    for (semantic, accessor) in primitive.attributes() {
        mesh_data.vertex_data_from_accessor(semantic, accessor, &buffers);
    }

    (mesh_data.0, mesh_data.1)
}

#[derive(Default)]
pub struct MeshData(pub VertexData, pub IndexData);

impl MeshData {
    const VEC3_SIZE: usize = 12;
    const VEC4_SIZE: usize = 16;
    const INDEX_SIZE: usize = 4;

    pub fn vertex_data_from_accessor(
        &mut self,
        semantic: Semantic,
        accessor: Accessor,
        buffers: &[buffer::Data],
    ) {
        let dst_offset = match semantic {
            Semantic::Positions => 0,
            Semantic::Normals => Self::VEC4_SIZE,
            _ => return,
        };

        let value_size = accessor.size();
        let count = accessor.count();
        let mut src_offset = accessor.offset();

        self.0.resize(count * Self::VEC4_SIZE * 2, 0);

        let view = accessor.view().unwrap();
        src_offset += view.offset();
        let src_step = view.stride().unwrap_or(Self::VEC3_SIZE);
        let dst_step = view.stride().unwrap_or(Self::VEC4_SIZE) * 2;

        let buffer_index = view.buffer().index();
        let buffer = &buffers[buffer_index];

        let src = &buffer[src_offset..(src_offset + count * src_step)];
        let dst = &mut self.0[dst_offset..];
        copy_with_step(src, src_step, dst, dst_step, value_size);
    }

    pub fn index_data_from_accessor(&mut self, accessor: Accessor, buffers: &[buffer::Data]) {
        let mut src_offset = accessor.offset();
        let count = accessor.count();

        let view = accessor.view().unwrap();
        src_offset += view.offset();

        let stride = view.stride();
        let buffer_index = view.buffer().index();
        let buffer = &buffers[buffer_index];

        let data_len = count * Self::INDEX_SIZE;
        self.1.resize(data_len, 0);
        match stride {
            Some(stride) => {
                let src = &buffer[src_offset..(src_offset + data_len)];
                let dst = self.1.as_mut_slice();
                copy_with_step(src, stride, dst, Self::INDEX_SIZE, Self::INDEX_SIZE);
            }
            None => self
                .1
                .copy_from_slice(&buffer[src_offset..(src_offset + data_len)]),
        }
    }
}

fn copy_with_step(src: &[u8], src_step: usize, dst: &mut [u8], dst_step: usize, value_size: usize) {
    let count = src.len() / src_step;
    for i in 0..count {
        let src_start = i * src_step;
        let src_end = i * src_step + value_size;
        let dst_start = i * dst_step;
        let dst_end = i * dst_step + value_size;
        dst[dst_start..dst_end].copy_from_slice(&src[src_start..src_end]);
    }
}

fn main() {}
