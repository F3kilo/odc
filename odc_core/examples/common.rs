#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use fps_counter::FPSCounter;
use gltf::{buffer, Accessor, Semantic};
use odc_core::mdl::{RenderModel, Size2d};
use odc_core::{DrawData, DrawDataSource, OdcCore, Stage, WindowInfo};
use std::mem;
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
    fn draw_stages(&self) -> Vec<Stage>;
    fn draw_data(&self) -> DrawDataTree;
}

pub fn run_example<E: Example + 'static>(mut ex: E) -> ! {
    env_logger::init();
    let event_loop = EventLoop::new();
    let color_window = winit::window::Window::new(&event_loop).unwrap();
    let window_info = WindowInfo {
        name: "color_window",
        handle: &color_window,
        size: Size2d {
            x: color_window.inner_size().width as _,
            y: color_window.inner_size().height as _,
        },
    };

    let depth_window = winit::window::Window::new(&event_loop).unwrap();
    let depth_window_info = WindowInfo {
        name: "depth_window",
        handle: &depth_window,
        size: Size2d {
            x: color_window.inner_size().width as _,
            y: color_window.inner_size().height as _,
        },
    };

    let mut renderer = OdcCore::with_window_support(E::render_model(), &color_window);
    let mut fps_counter = FPSCounter::new();
    let color_source = 0;
    let depth_source = 1;
    unsafe { renderer.add_window(color_source, window_info) };
    unsafe { renderer.add_window(depth_source, depth_window_info) };
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
                window_id,
            } => {
                let size = Size2d {
                    x: size.width as _,
                    y: size.height as _,
                };
                if window_id == color_window.id() {
                    renderer.resize_window("color_window", size);
                    renderer.resize_attachments(color_source, size);
                }

                if window_id == depth_window.id() {
                    renderer.resize_window("depth_window", size);
                }
            }
            Event::MainEventsCleared => {
                let stages = ex.draw_stages();
                let data = ex.draw_data();
                renderer.draw(&data, &stages);
                let fps = fps_counter.tick();
                color_window.set_title(&format!("FPS: {}", fps));
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                renderer.remove_window("color_window");
                renderer.remove_window("depth_window");
                *flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

pub struct DrawDataTree(pub Vec<Vec<Vec<DrawData>>>);

impl DrawDataSource for DrawDataTree {
    fn draw_data(&self, pass: usize, pipeline: usize) -> &[DrawData] {
        &self.0[pass][pipeline]
    }
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
